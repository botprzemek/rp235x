use defmt::*;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_rp::rom_data::reset_to_usb_boot;

#[embassy_executor::task]
pub async fn udp_task(stack: embassy_net::Stack<'static>) {
    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut rx_buffer = [0; 4096];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );
    socket.bind(1337).unwrap();

    loop {
        let (n, ep) = unwrap!(socket.recv_from(&mut buf).await);

        if let Ok(s) = core::str::from_utf8(&buf[..n]) {
            log::info!("ECHO (to {}): {}", ep, s);

            if s == "BOOTSEL" {
                reset_to_usb_boot(0, 0);
                break;
            }
        } else {
            log::info!("ECHO (to {}): bytearray len {}", ep, n);
        }

        unwrap!(socket.send_to(&buf[..n], ep).await);
    }
}
