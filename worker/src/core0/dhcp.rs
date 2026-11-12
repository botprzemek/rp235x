use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{Ipv4Address, Stack};

pub async fn run_dhcp_server(stack: Stack<'static>) -> ! {
    let mut rx_meta = [PacketMetadata::EMPTY; 4];
    let mut rx_buf = [0u8; 1024];
    let mut tx_meta = [PacketMetadata::EMPTY; 4];
    let mut tx_buf = [0u8; 1024];

    let mut socket = UdpSocket::new(stack, &mut rx_meta, &mut rx_buf, &mut tx_meta, &mut tx_buf);

    if let Err(e) = socket.bind(67) {
        defmt::error!("Failed to bind DHCP socket on port 67: {:?}", e);
        loop {
            embassy_time::Timer::after_secs(60).await;
        }
    }

    defmt::info!("DHCP server listening on 0.0.0.0:67");

    let server_ip = [192, 168, 4, 1];
    let offered_ip = [192, 168, 4, 2];
    let subnet_mask = [255, 255, 255, 0];

    let mut packet_buf = [0u8; 1024];

    loop {
        let (n, _remote_endpoint) = match socket.recv_from(&mut packet_buf).await {
            Ok(res) => res,
            Err(e) => {
                defmt::warn!("DHCP recv error: {:?}", e);
                continue;
            }
        };

        if n < 240 {
            continue;
        }

        if packet_buf[0] != 1 {
            continue;
        }

        let xid = &packet_buf[4..8];
        let chaddr = &packet_buf[28..44];

        if packet_buf[236..240] != [0x63, 0x82, 0x53, 0x63] {
            continue;
        }

        let mut msg_type = 0u8;
        let mut idx = 240;
        while idx < n {
            let opt = packet_buf[idx];
            if opt == 255 {
                break;
            }
            if opt == 0 {
                idx += 1;
                continue;
            }
            if idx + 1 >= n {
                break;
            }
            let len = packet_buf[idx + 1] as usize;
            if idx + 2 + len > n {
                break;
            }
            if opt == 53 && len == 1 {
                msg_type = packet_buf[idx + 2];
            }
            idx += 2 + len;
        }

        let reply_type = match msg_type {
            1 => 2,
            3 => 5,
            _ => continue,
        };

        defmt::info!(
            "DHCP incoming type {}, replying with {}",
            msg_type,
            reply_type
        );

        let mut resp = [0u8; 300];
        resp[0] = 2;
        resp[1] = 1;
        resp[2] = 6;
        resp[3] = 0;
        resp[4..8].copy_from_slice(xid);
        resp[8..10].copy_from_slice(&[0, 0]);
        resp[10..12].copy_from_slice(&[0x80, 0x00]);

        resp[16..20].copy_from_slice(&offered_ip);

        resp[20..24].copy_from_slice(&server_ip);

        resp[28..44].copy_from_slice(chaddr);

        resp[236..240].copy_from_slice(&[0x63, 0x82, 0x53, 0x63]);

        let mut opt_idx = 240;

        resp[opt_idx..opt_idx + 3].copy_from_slice(&[53, 1, reply_type]);
        opt_idx += 3;

        resp[opt_idx..opt_idx + 6].copy_from_slice(&[
            54,
            4,
            server_ip[0],
            server_ip[1],
            server_ip[2],
            server_ip[3],
        ]);
        opt_idx += 6;

        resp[opt_idx..opt_idx + 6].copy_from_slice(&[51, 4, 0x00, 0x01, 0x51, 0x80]);
        opt_idx += 6;

        resp[opt_idx..opt_idx + 6].copy_from_slice(&[
            1,
            4,
            subnet_mask[0],
            subnet_mask[1],
            subnet_mask[2],
            subnet_mask[3],
        ]);
        opt_idx += 6;

        resp[opt_idx..opt_idx + 6].copy_from_slice(&[
            3,
            4,
            server_ip[0],
            server_ip[1],
            server_ip[2],
            server_ip[3],
        ]);
        opt_idx += 6;

        resp[opt_idx..opt_idx + 6].copy_from_slice(&[
            6,
            4,
            server_ip[0],
            server_ip[1],
            server_ip[2],
            server_ip[3],
        ]);
        opt_idx += 6;

        resp[opt_idx] = 255;
        opt_idx += 1;

        let broadcast_endpoint = embassy_net::IpEndpoint {
            addr: embassy_net::IpAddress::Ipv4(Ipv4Address::new(255, 255, 255, 255)),
            port: 68,
        };

        if let Err(e) = socket.send_to(&resp[..opt_idx], broadcast_endpoint).await {
            defmt::warn!("DHCP send error: {:?}", e);
        }
    }
}
