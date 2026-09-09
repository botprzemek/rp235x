use crate::peripherals::{NetPeripherals, TrngPeripherals};
use crate::state::{Input, Machine};
use crate::{
    core0::{
        cyw43::{Cyw43, cyw43_task},
        net::{Net, net_task},
    },
    state::GAME_STATE,
};
use config::{Config, print::Print};
use defmt::{error, info};
use defmt::{println, unwrap};
use embassy_executor::Spawner;
use embassy_net::{
    Stack,
    udp::{PacketMetadata, UdpSocket},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use net::ServerPacket;
use net::data::snapshot::Snapshot;
use static_cell::StaticCell;

pub static CORE1_READY_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

static X_UDP_RX_BUFFER: StaticCell<[u8; 512]> = StaticCell::new();
static X_UDP_TX_BUFFER: StaticCell<[u8; 512]> = StaticCell::new();

pub trait Handler {
    async fn handle_boot(&mut self) -> ();
    async fn handle_sync(&mut self) -> ();
    async fn handle_networking(
        &mut self,
        spawner: Spawner,
        net: NetPeripherals,
        trng: TrngPeripherals,
    ) -> ();
    async fn _handle_running(&mut self) -> ();
    async fn _handle_error_recovery(&mut self) -> ();
}

impl Handler for Machine {
    async fn handle_boot(&mut self) {
        let config = match Config::read() {
            Ok(config) => config,
            Err(_) => return self.transition(Input::BootFailed),
        };

        if config.print().is_err() {
            return self.transition(Input::BootFailed);
        }

        self.transition(Input::BootSuccess);
    }

    async fn handle_sync(&mut self) {
        CORE1_READY_SIGNAL.wait().await;

        self.transition(Input::CoreSynced);
    }

    async fn handle_networking(
        &mut self,
        spawner: Spawner,
        net: NetPeripherals,
        trng: TrngPeripherals,
    ) {
        let seed = TrngPeripherals::init(trng).await;
        let (net_device, mut control, runner) = Cyw43::init(net).await;
        spawner.spawn(unwrap!(cyw43_task(runner)));

        let (stack, runner) = Net::init_stack(&mut control, net_device, seed).await;
        spawner.spawn(unwrap!(net_task(runner)));

        let config = Config::read().unwrap();
        Net::init_wifi(&mut control, config).await;
        stack.wait_link_up().await;
        stack.wait_config_up().await;

        match stack.config_v4() {
            Some(config_v4) => {
                defmt::info!(
                    "NetworkConfig::Address               {}",
                    &config_v4.address
                );
                defmt::info!(
                    "NetworkConfig::Gateway               {}",
                    &config_v4.gateway.unwrap()
                );

                self.transition(Input::WifiConnected);
            }
            None => self.transition(Input::WifiFailed),
        }

        spawner.spawn(unwrap!(udp_task(stack)));
    }

    async fn _handle_running(&mut self) {}

    async fn _handle_error_recovery(&mut self) {
        embassy_time::Timer::after_secs(1).await;
    }
}

use net::{ClientPacket, event::ClientEvent};
#[embassy_executor::task]
pub async fn udp_task(stack: Stack<'static>) {
    let rx_buffer = X_UDP_RX_BUFFER.init([0; 512]);
    let tx_buffer = X_UDP_TX_BUFFER.init([0; 512]);

    let mut rx_meta = [PacketMetadata::EMPTY; 2];
    let mut tx_meta = [PacketMetadata::EMPTY; 2];

    let mut socket = UdpSocket::new(stack, &mut rx_meta, rx_buffer, &mut tx_meta, tx_buffer);

    if let Err(e) = socket.bind(8001) {
        error!("socket_error: {:?}", e);
        return;
    }

    let broadcast_endpoint =
        embassy_net::IpEndpoint::new(embassy_net::IpAddress::v4(255, 255, 255, 255), 8000);

    let mut rx_packet_buf = [0u8; net::layout::PACKET_SIZE];

    loop {
        info!("[WORKER] Szukanie mastera w sieci (Broadcast)...");
        let mut master_endpoint: Option<embassy_net::IpEndpoint> = None;

        while master_endpoint.is_none() {
            let mocked_data = [0u8; 27];
            let client_packet = ClientPacket::new(ClientEvent::HandshakeRequest, 0, mocked_data);
            let packet_bytes = client_packet.to_bytes();

            if let Err(e) = socket.send_to(&packet_bytes, broadcast_endpoint).await {
                error!("broadcast_send_error: {:?}", e);
            }

            let recv_future = socket.recv_from(&mut rx_packet_buf);
            let (size, remote_endpoint) = match embassy_time::with_timeout(
                embassy_time::Duration::from_millis(500),
                recv_future,
            )
            .await
            {
                Ok(Ok((size, remote_endpoint))) => (size, remote_endpoint),
                Ok(Err(e)) => {
                    info!("{}", e);
                    continue;
                }
                Err(e) => {
                    info!("{}", e);
                    continue;
                }
            };

            if size != net::layout::PACKET_SIZE {
                continue;
            }

            let _packet = match ServerPacket::from_bytes(&rx_packet_buf) {
                Ok(packet) => packet,
                Err(_) => continue,
            };

            info!(
                "[WORKER] Znaleziono mastera pod adresem: {}. Handshake zakończony!",
                remote_endpoint
            );
            master_endpoint = Some(remote_endpoint.endpoint);
        }

        let master_endpoint = master_endpoint.unwrap();
        info!("[WORKER] Przejście do głównej pętli odbierania danych.");

        let mut keepalive_timer = embassy_time::Ticker::every(embassy_time::Duration::from_secs(3));
        let mut missed_heartbeats: u8 = 0;
        const MAX_MISSED_HEARTS: u8 = 3;

        loop {
            let recv_fut = socket.recv_from(&mut rx_packet_buf);
            let tick_fut = keepalive_timer.next();

            match embassy_futures::select::select(recv_fut, tick_fut).await {
                embassy_futures::select::Either::First(res) => match res {
                    Ok((size, remote_endpoint)) => {
                        if size != net::layout::PACKET_SIZE {
                            continue;
                        }

                        if remote_endpoint.endpoint.addr != master_endpoint.addr {
                            continue;
                        }

                        missed_heartbeats = 0;

                        let packet = match ServerPacket::from_bytes(&rx_packet_buf) {
                            Ok(packet) => packet,
                            Err(_) => continue,
                        };

                        let game_ref = GAME_STATE.lock().await;
                        *game_ref.borrow_mut() = Snapshot::try_from(packet).unwrap();
                    }
                    Err(e) => {
                        error!("udp_rx_error: {:?}", e);
                    }
                },
                embassy_futures::select::Either::Second(_) => {
                    let mocked_data = [0u8; 27];
                    let keepalive_packet = ClientPacket::new(ClientEvent::Start, 0, mocked_data);
                    let packet_bytes = keepalive_packet.to_bytes();

                    if let Err(e) = socket.send_to(&packet_bytes, master_endpoint).await {
                        error!("keepalive_send_error: {:?}", e);
                    } else {
                        info!("[WORKER] Wysłano keepalive do mastera");

                        missed_heartbeats += 1;
                        if missed_heartbeats < MAX_MISSED_HEARTS {
                            continue;
                        }

                        info!("[WORKER] Brak odpowiedzi od Mastera przez 9s! Utracono połączenie.");

                        break;
                    }
                }
            }
        }
    }
}
