pub mod cyw43;
pub mod net;

use crate::handler::Handler;
use crate::peripherals::{NetPeripherals, TrngPeripherals};
use crate::state::{Machine, State};
use defmt::unwrap;
use embassy_executor::{Executor, Spawner};
use static_cell::StaticCell;

pub static EXECUTOR: StaticCell<Executor> = StaticCell::new();

pub struct Core0;

impl Core0 {
    pub fn entry(net: NetPeripherals, trng: TrngPeripherals) -> ! {
        let executor = Executor::new();

        EXECUTOR.init(executor).run(|spawner| {
            spawner.spawn(unwrap!(task(spawner, net, trng)));
        })
    }
}

#[embassy_executor::task]
async fn task(spawner: Spawner, net: NetPeripherals, trng: TrngPeripherals) {
    let mut state_machine = Machine::new();

    state_machine.handle_boot().await;
    state_machine.handle_sync().await;
    state_machine.handle_networking(spawner, net, trng).await;

    loop {
        match state_machine.current_state() {
            State::Running => state_machine.handle_rest().await,
            _ => state_machine.handle_rest().await,
        }
    }
}

// static SOCKETS: StaticCell<embassy_net::StackResources<5>> = StaticCell::new();

// static X_UDP_RX_BUFFER: StaticCell<[u8; 512]> = StaticCell::new();
// static X_UDP_TX_BUFFER: StaticCell<[u8; 512]> = StaticCell::new();

// #[embassy_executor::task]
// pub async fn game_task(stack: Stack<'static>) {
//     let rx_buffer = X_UDP_RX_BUFFER.init([0; 512]);
//     let tx_buffer = X_UDP_TX_BUFFER.init([0; 512]);

//     let mut rx_meta = [PacketMetadata::EMPTY; 2];
//     let mut tx_meta = [PacketMetadata::EMPTY; 2];

//     let mut socket = UdpSocket::new(stack, &mut rx_meta, rx_buffer, &mut tx_meta, tx_buffer);

//     // Bindowanie lokalnego portu (0 oznacza losowy wolny port wychodzący)
//     if let Err(e) = socket.bind(12346) {
//         defmt::error!("Nie udało się przypisać portu UDP: {:?}", e);
//         return;
//     }

//     defmt::info!("Zadanie sieciowe uruchomione. Klient: 192.168.0.102:12345");

//     let sender = shared::channels::GAME_CHANNEL.sender();
//     // Bufor na pojedynczy przychodzący pakiet (dokładnie PACKET_SIZE = 27 bajtów)
//     let mut rx_packet_buf = [0u8; game_net::PACKET_SIZE];

//     loop {
//         // Asynchroniczne oczekiwanie na dowolny pakiet UDP
//         match socket.recv_from(&mut rx_packet_buf).await {
//             Ok((size, _remote_endpoint)) => {
//                 // Sprawdzamy, czy pakiet ma odpowiednią długość protokołu
//                 if size != game_net::PACKET_SIZE {
//                     defmt::warn!(
//                         "Odrzucono pakiet o nieprawidłowym rozmiarze: {} bajtów",
//                         size
//                     );
//                     continue;
//                 }

//                 // Próba deserializacji pakietu i weryfikacji sumy kontrolnej CRC
//                 match game_net::Packet::from_bytes(&rx_packet_buf) {
//                     Ok(packet) => {
//                         sender.send(packet).await;
//                     }
//                     Err(_) => {
//                         defmt::error!("Błąd walidacji pakietu (złe MAGIC lub suma CRC)");
//                     }
//                 }
//             }
//             Err(e) => {
//                 defmt::error!("Błąd podczas odbierania UDP: {:?}", e);
//             }
//         }
//     }

// #[embassy_executor::task]
// pub async fn task(
//     spawner: embassy_executor::Spawner,
//     net_peripherals: NetPeripherals,
//     trng_peripherals: TrngPeripherals,
// ) {
//     let seed = Net::init_trng(trng_peripherals).await;
//     let (net_device, mut control, runner) = Net::init_cyw43(net_peripherals).await;

//     spawner.spawn(unwrap!(cyw43_task(runner)));

//     let (stack, runner) = Net::init_stack(seed, net_device, &mut control).await;

//     spawner.spawn(unwrap!(net_task(runner)));

//     Net::init_wifi(&mut control).await;
//     stack.wait_link_up().await;
//     stack.wait_config_up().await;

//     if let Some(config) = stack.config_v4() {
//         info!("net::stack up");
//         info!("   ::ip      {}", config.address.address());

//         if let Some(gateway) = config.gateway {
//             info!("   ::gateway {}", gateway);
//         }
//     }

//     spawner.spawn(unwrap!(game_task(stack)));
// }
