//! Inherent state handlers on [`Machine`] and the UDP client task.

use basketball::Game;
use config::Config;
use config::flash::FlashReader;
use config::print::Print;
use embassy_executor::Spawner;
use embassy_futures::select::{self, Either};
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{IpAddress, IpEndpoint, Stack};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Ticker, Timer};
use net::event::ClientEvent;
use net::layout::PACKET_SIZE;
use net::{ClientPacket, Device, DeviceType, ServerPacket};
use static_cell::StaticCell;

use crate::core0::cyw43::{Cyw43, cyw43_task};
use crate::core0::net::{Net, net_task};
use crate::peripherals::{NetPeripherals, TrngPeripherals};
use crate::state::{GAME_STATE, Input, Machine};

/// Signal emitted by Core 1 once its HUB75 display task has been spawned.
pub static CORE1_READY_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

static UDP_RX_BUFFER: StaticCell<[u8; 512]> = StaticCell::new();
static UDP_TX_BUFFER: StaticCell<[u8; 512]> = StaticCell::new();

impl Machine {
    /// Verifies and prints the flash configuration during boot.
    pub async fn handle_boot(&mut self) {
        let Ok(config) = Config::read() else {
            self.transition(Input::BootFailed);
            return;
        };

        if config.print().is_err() {
            self.transition(Input::BootFailed);
            return;
        }

        self.transition(Input::BootSuccess);
    }

    /// Waits for Core 1 to signal readiness.
    pub async fn handle_sync(&mut self) {
        CORE1_READY_SIGNAL.wait().await;

        self.transition(Input::CoreSynced);
    }

    /// Initializes the TRNG, CYW43 Wi-Fi chip, DHCPv4 stack, and UDP synchronization task.
    pub async fn handle_networking(
        &mut self,
        spawner: Spawner,
        net: NetPeripherals,
        trng: TrngPeripherals,
    ) {
        let seed = TrngPeripherals::init(trng).await;
        let mut cyw43_driver = Cyw43::init(net).await;
        spawner.spawn(defmt::unwrap!(cyw43_task(cyw43_driver.runner)));

        let initialized_net =
            Net::init_stack(&mut cyw43_driver.control, cyw43_driver.net_driver, seed).await;
        spawner.spawn(defmt::unwrap!(net_task(initialized_net.runner)));

        let config = Config::read().expect("verified flash config must be readable");
        Net::init_wifi(&mut cyw43_driver.control, config).await;
        initialized_net.stack.wait_link_up().await;
        initialized_net.stack.wait_config_up().await;

        match initialized_net.stack.config_v4() {
            Some(config_v4) => {
                defmt::info!(
                    "NetworkConfig::Address               {}",
                    &config_v4.address
                );
                if let Some(ref gateway) = config_v4.gateway {
                    defmt::info!("NetworkConfig::Gateway               {}", gateway);
                }

                self.transition(Input::WifiConnected);
            }
            None => self.transition(Input::WifiFailed),
        }

        spawner.spawn(defmt::unwrap!(udp_task(initialized_net.stack)));
    }

    #[allow(dead_code)]
    async fn handle_running(&mut self) {}

    #[allow(dead_code)]
    async fn handle_error_recovery(&mut self) {
        Timer::after_secs(1).await;
    }
}

/// Background Embassy task discovering the master server over UDP and receiving 60 Hz game frames.
#[embassy_executor::task]
pub async fn udp_task(stack: Stack<'static>) {
    let receive_buffer = UDP_RX_BUFFER.init([0; 512]);
    let transmit_buffer = UDP_TX_BUFFER.init([0; 512]);

    let mut receive_metadata = [PacketMetadata::EMPTY; 2];
    let mut transmit_metadata = [PacketMetadata::EMPTY; 2];

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

    let config = Config::read().expect("verified flash config must be readable");
    let device_bytes = Device::from_config(DeviceType::Worker, config)
        .expect("flash config must produce a valid Worker Device")
        .to_bytes();

    let broadcast_endpoint = IpEndpoint::new(IpAddress::v4(255, 255, 255, 255), 9000);

    let mut packet_buffer = [0u8; PACKET_SIZE];

    loop {
        defmt::info!("[WORKER] Szukanie mastera w sieci (Broadcast)...");

        let master_endpoint = loop {
            let client_packet = ClientPacket::new(ClientEvent::HandshakeRequest, 0, device_bytes);
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

            if size != PACKET_SIZE {
                continue;
            }

            if ServerPacket::from_bytes(packet_buffer).is_err() {
                continue;
            }

            defmt::info!(
                "[WORKER] Znaleziono mastera pod adresem: {}. Handshake zakończony!",
                remote_endpoint
            );
            break remote_endpoint.endpoint;
        };

        defmt::info!("[WORKER] Przejście do głównej pętli odbierania danych.");

        let mut keepalive_timer = Ticker::every(Duration::from_secs(3));
        let mut missed_heartbeats: u8 = 0;
        const MAX_MISSED_HEARTBEATS: u8 = 3;

        loop {
            let receive_future = socket.recv_from(&mut packet_buffer);
            let tick_future = keepalive_timer.next();

            match select::select(receive_future, tick_future).await {
                Either::First(result) => match result {
                    Ok((size, remote_endpoint)) => {
                        if size != PACKET_SIZE {
                            continue;
                        }

                        if remote_endpoint.endpoint.addr != master_endpoint.addr {
                            continue;
                        }

                        missed_heartbeats = 0;

                        let Ok(packet) = ServerPacket::from_bytes(packet_buffer) else {
                            continue;
                        };

                        if let Ok(game) = Game::try_from(packet) {
                            let game_ref = GAME_STATE.lock().await;
                            *game_ref.borrow_mut() = game;
                        }
                    }
                    Err(error) => {
                        defmt::error!("udp_rx_error: {:?}", error);
                    }
                },
                Either::Second(()) => {
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
