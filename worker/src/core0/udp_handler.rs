//! Unified UDP handler for Worker state synchronization and OTA Wi-Fi configuration.

use crate::peripherals::flash::write_config_to_flash;
use crate::state::GAME_STATE;

use basketball::Game;
use config::flash::FlashReader;
use config::{Config, ConfigBuilder};
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{IpAddress, IpEndpoint, Stack};
use embassy_rp::peripherals::{FLASH, WATCHDOG};
use embassy_rp::watchdog::Watchdog;
use embassy_rp::Peri;
use embassy_time::{Duration, Ticker, Timer};
use net::event::{ClientEvent, ServerEvent};
use net::layout::PACKET_SIZE;
use net::{ClientPacket, Device, DeviceConfigPacket, DeviceType, ServerPacket};
use static_cell::StaticCell;

static UDP_RX_BUFFER: StaticCell<[u8; 1024]> = StaticCell::new();
static UDP_TX_BUFFER: StaticCell<[u8; 1024]> = StaticCell::new();

/// Background Embassy task running on port 8001.
///
/// Handles:
/// 1. `ClientEvent::DeviceConfigSet` - OTA Wi-Fi configuration updates over UDP.
///    Sends `ServerEvent::DeviceConfigAck`, writes to flash, sends `ServerEvent::DeviceConfigSuccess`,
///    and triggers a clean reset via Watchdog.
/// 2. In STA mode, discovers the master server over broadcast on port 9000, maintains heartbeats,
///    and receives game frames at 60 Hz.
#[embassy_executor::task]
pub async fn unified_udp_task(
    stack: Stack<'static>,
    watchdog_peri: Peri<'static, WATCHDOG>,
    mut flash_peri: Peri<'static, FLASH>,
    is_ap_mode: bool,
) {
    let receive_buffer = UDP_RX_BUFFER.init([0; 1024]);
    let transmit_buffer = UDP_TX_BUFFER.init([0; 1024]);

    let mut receive_metadata = [PacketMetadata::EMPTY; 4];
    let mut transmit_metadata = [PacketMetadata::EMPTY; 4];

    let mut socket = UdpSocket::new(
        stack,
        &mut receive_metadata,
        receive_buffer,
        &mut transmit_metadata,
        transmit_buffer,
    );

    if let Err(error) = socket.bind(8001) {
        defmt::error!("socket_error: {:?}", error);
        return;
    }

    defmt::info!("[WORKER UDP] Bound to port 8001 (AP Mode: {})", is_ap_mode);

    let mut watchdog = Watchdog::new(watchdog_peri);
    let mut packet_buffer = [0u8; 512];

    if is_ap_mode {
        // In AP mode, worker simply waits for DeviceConfigPacket or DeviceConfigSet packets on port 8001
        loop {
            let (size, remote_endpoint) = match socket.recv_from(&mut packet_buffer).await {
                Ok(res) => res,
                Err(e) => {
                    defmt::warn!("UDP recv error: {:?}", e);
                    continue;
                }
            };

            if let Ok(config_pkt) = DeviceConfigPacket::from_bytes(&packet_buffer[..size]) {
                handle_device_config(
                    &mut socket,
                    remote_endpoint,
                    config_pkt,
                    flash_peri.reborrow(),
                    &mut watchdog,
                )
                .await;
            }
        }
    } else {
        // In STA mode, discover master server and sync game state, while also accepting DeviceConfigSet
        let loaded_config = Config::read().ok();
        let device_bytes = loaded_config
            .and_then(|c| Device::from_config(DeviceType::Worker, c).ok())
            .unwrap_or_else(|| {
                Device::try_new(DeviceType::Worker, "sn-hmi-0000", "hmi-worker")
                    .expect("valid default worker device")
            })
            .to_bytes();

        let broadcast_endpoint = IpEndpoint::new(IpAddress::v4(255, 255, 255, 255), 9000);

        loop {
            defmt::info!("[WORKER] Szukanie mastera w sieci (Broadcast)...");

            let (master_endpoint, mut session_token) = loop {
                let client_packet =
                    ClientPacket::new(ClientEvent::HandshakeRequest, 0, device_bytes);
                let packet_bytes = client_packet.to_bytes();

                if let Err(error) = socket.send_to(&packet_bytes, broadcast_endpoint).await {
                    defmt::error!("broadcast_send_error: {:?}", error);
                }

                let receive_future = socket.recv_from(&mut packet_buffer);
                let (size, remote_endpoint) = match embassy_time::with_timeout(
                    Duration::from_millis(500),
                    receive_future,
                )
                .await
                {
                    Ok(Ok((size, remote_endpoint))) => (size, remote_endpoint),
                    Ok(Err(error)) => {
                        defmt::info!("{}", error);
                        continue;
                    }
                    Err(timeout_error) => {
                        defmt::info!("{}", timeout_error);
                        continue;
                    }
                };

                // Check if this is an incoming DeviceConfigPacket while waiting for master
                if let Ok(config_pkt) = DeviceConfigPacket::from_bytes(&packet_buffer[..size]) {
                    handle_device_config(
                        &mut socket,
                        remote_endpoint,
                        config_pkt,
                        flash_peri.reborrow(),
                        &mut watchdog,
                    )
                    .await;
                    continue;
                }

                if size != PACKET_SIZE {
                    continue;
                }

                let Ok(server_packet) = ServerPacket::from_bytes(
                    packet_buffer[..PACKET_SIZE]
                        .try_into()
                        .unwrap_or([0; PACKET_SIZE]),
                ) else {
                    continue;
                };

                let session_token = if let Ok(game) = Game::try_from(server_packet) {
                    GAME_STATE.set(game);
                    game.session_token()
                } else {
                    0
                };

                defmt::info!(
                    "[WORKER] Znaleziono mastera pod adresem: {}. Handshake zakończony (Token: {:x})!",
                    remote_endpoint,
                    session_token
                );
                break (remote_endpoint.endpoint, session_token);
            };

            defmt::info!("[WORKER] Przejście do głównej pętli odbierania danych.");

            let mut keepalive_timer = Ticker::every(Duration::from_secs(3));
            let mut missed_heartbeats: u8 = 0;
            const MAX_MISSED_HEARTBEATS: u8 = 3;

            loop {
                let receive_future = socket.recv_from(&mut packet_buffer);
                let tick_future = keepalive_timer.next();

                match embassy_futures::select::select(receive_future, tick_future).await {
                    embassy_futures::select::Either::First(result) => match result {
                        Ok((size, remote_endpoint)) => {
                            // Check for OTA Wi-Fi configuration packet from HMI
                            if let Ok(config_pkt) = DeviceConfigPacket::from_bytes(&packet_buffer[..size]) {
                                handle_device_config(
                                    &mut socket,
                                    remote_endpoint,
                                    config_pkt,
                                    flash_peri.reborrow(),
                                    &mut watchdog,
                                )
                                .await;
                                continue;
                            }

                            if size != PACKET_SIZE {
                                continue;
                            }

                            if remote_endpoint.endpoint.addr != master_endpoint.addr {
                                continue;
                            }

                            missed_heartbeats = 0;

                            let Ok(packet) = ServerPacket::from_bytes(packet_buffer[..PACKET_SIZE].try_into().unwrap_or([0; PACKET_SIZE])) else {
                                continue;
                            };

                            if let Ok(game) = Game::try_from(packet) {
                                if packet.event() == ServerEvent::HandshakeAck {
                                    session_token = game.session_token();
                                    GAME_STATE.set(game);
                                } else if session_token == 0 || game.session_token() == 0 || game.session_token() == session_token {
                                    GAME_STATE.set(game);
                                } else {
                                    defmt::warn!("[WORKER] Odrzucono pakiet UDP: nieprawidłowy token sesji!");
                                }
                            }
                        }
                        Err(error) => {
                            defmt::error!("udp_rx_error: {:?}", error);
                        }
                    },
                    embassy_futures::select::Either::Second(()) => {
                        let keepalive_packet =
                            ClientPacket::new(ClientEvent::HandshakeHeartbeat, 0, device_bytes);
                        let packet_bytes = keepalive_packet.to_bytes();

                        if let Err(error) = socket.send_to(&packet_bytes, master_endpoint).await {
                            defmt::error!("keepalive_send_error: {:?}", error);
                        } else {
                            defmt::info!("[WORKER] Wysłano keepalive do mastera");

                            missed_heartbeats += 1;
                            if missed_heartbeats < MAX_MISSED_HEARTBEATS {
                                continue;
                            }

                            defmt::info!(
                                "[WORKER] Brak odpowiedzi od Mastera przez 9s! Utracono połączenie."
                            );

                            break;
                        }
                    }
                }
            }
        }
    }
}

async fn handle_device_config(
    socket: &mut UdpSocket<'_>,
    remote_endpoint: embassy_net::udp::UdpMetadata,
    packet: DeviceConfigPacket,
    flash_peri: Peri<'_, FLASH>,
    watchdog: &mut Watchdog,
) {
    defmt::info!("[WORKER] Received DeviceConfigPacket from {}", remote_endpoint);

    // 1. Immediately send Ack
    let ack_packet = ServerPacket::new(ServerEvent::DeviceConfigAck, 0, [0u8; net::layout::DATA_SIZE]);
    let _ = socket.send_to(&ack_packet.to_bytes(), remote_endpoint.endpoint).await;

    let Ok(sn) = packet.serial_number() else { return; };
    let Ok(id) = packet.device_id() else { return; };
    let Ok(ssid) = packet.wifi_ssid() else { return; };
    let Ok(pass) = packet.wifi_password() else { return; };

    defmt::info!("[WORKER] Applying config: SN='{}', ID='{}', SSID='{}'", sn, id, ssid);

    let mut builder = ConfigBuilder::new();
    builder.serial_number(sn).device_id(id).wifi_ssid(ssid).wifi_password(pass);

    let Ok(new_config) = builder.build() else {
        defmt::error!("[WORKER] Invalid Config fields");
        return;
    };

    if write_config_to_flash(flash_peri, &new_config).is_ok() {
        defmt::info!("[WORKER] Flash programmed successfully! Sending DeviceConfigSuccess...");

        // 2. Send Success packet back to HMI
        let success_packet = ServerPacket::new(
            ServerEvent::DeviceConfigSuccess,
            0,
            [0u8; net::layout::DATA_SIZE],
        );
        let _ = socket.send_to(&success_packet.to_bytes(), remote_endpoint.endpoint).await;

        Timer::after_millis(300).await;
        defmt::info!("[WORKER] Rebooting via Watchdog into new network...");
        watchdog.trigger_reset();
    } else {
        defmt::error!("[WORKER] Failed to write config to flash!");
    }
}
