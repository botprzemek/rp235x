use cyw43::Control;
use defmt::*;
use embassy_net::Ipv4Address;
use embassy_net::tcp::TcpSocket;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_rp::rom_data::reset_to_usb_boot;
use embassy_time::{Duration, Timer};
use embedded_io_async::Write;

#[embassy_executor::task]
pub async fn udp_task(stack: embassy_net::Stack<'static>) {
    log::info!("udp listen");

    let mut rx_meta = [PacketMetadata::EMPTY; 2];
    let mut rx_buffer = [0; 256];
    let mut tx_meta = [PacketMetadata::EMPTY; 2];
    let mut tx_buffer = [0; 256];
    let mut buf = [0; 16];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );
    socket.bind(3000).unwrap();

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

#[embassy_executor::task]
pub async fn ssdp_task(stack: embassy_net::Stack<'static>) {
    log::info!("ssdp listen");

    let mut rx_meta = [PacketMetadata::EMPTY; 2];
    let mut rx_buffer = [0; 256];
    let mut tx_meta = [PacketMetadata::EMPTY; 2];
    let mut tx_buffer = [0; 512];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );

    unwrap!(socket.bind(0));

    let remote_endpoint = (Ipv4Address::new(239, 255, 255, 250), 1900);
    let ipv4_address = stack.config_v4().map(|c| c.address.address());

    let mut msg = heapless::String::<512>::new();
    let _ = core::fmt::write(
        &mut msg,
        format_args!(
            "NOTIFY * HTTP/1.1\r\n\
        HOST: 239.255.255.250:1900\r\n\
        CACHE-CONTROL: max-age=1800\r\n\
        LOCATION: http://{:?}:80/device.xml\r\n\
        NT: upnp:rootdevice\r\n\
        NTS: ssap:alive\r\n\
        SERVER: Tizen/5.5 UPnP/1.1 Samsung-Smart-Fridge/1.0\r\n\
        USN: uuid:30cda7ab-cdef-4940-8f1d-30cda7abcdef::upnp:rootdevice\r\n\r\n",
            ipv4_address
        ),
    );

    loop {
        let _ = socket.send_to(msg.as_bytes(), remote_endpoint).await;
        embassy_time::Timer::after_secs(30).await;
    }
}

#[embassy_executor::task]
pub async fn http_task(stack: embassy_net::Stack<'static>) {
    log::info!("http listen");

    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; 1536];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(5)));

        if socket.accept(80).await.is_err() {
            continue;
        }

        let mut request_buf = [0u8; 512];
        let _ = socket.read(&mut request_buf).await;

        let xml_body = "<?xml version=\"1.0\"?>\r\n\
            <root xmlns=\"urn:schemas-upnp-org:device-1-0\">\r\n\
            <specVersion><major>1</major><minor>0</minor></specVersion>\r\n\
            <device>\r\n\
            <deviceType>urn:schemas-upnp-org:device:Basic:1</deviceType>\r\n\
            <friendlyName>[Samsung] Family Hub</friendlyName>\r\n\
            <manufacturer>Samsung Electronics</manufacturer>\r\n\
            <modelName>RF28NHEDBSR</modelName>\r\n\
            <modelNumber>FamilyHub 2.0</modelNumber>\r\n\
            <serialNumber>30CDA7ABCDEF</serialNumber>\r\n\
            <UDN>uuid:30cda7ab-cdef-4940-8f1d-30cda7abcdef</UDN>\r\n\
            </device></root>\r\n";

        let mut response = heapless::String::<2048>::new();
        let _ = core::fmt::write(
            &mut response,
            format_args!(
                "HTTP/1.1 200 OK\r\n\
            Server: Samsung-Tizen-Web-Server\r\n\
            Content-Type: text/xml\r\n\
            Content-Length: {}\r\n\
            Connection: close\r\n\r\n\
            {}",
                xml_body.len(),
                xml_body
            ),
        );

        let _ = socket.write_all(response.as_bytes()).await;
        let _ = socket.flush().await;

        Timer::after_millis(200).await;
        socket.close();
    }
}
