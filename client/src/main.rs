use std::env;
use std::io::{self};
use std::net::{Ipv4Addr, UdpSocket};
use std::str;
use std::time::Duration;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return Ok(());
    }

    match args[1].as_str() {
        "reset" => {
            let target = args.get(2).expect("ip address eg. 192.168.0.105:3000");
            reset_pico(target)?;
        }
        "listen" => {
            listen_ssdp()?;
        }
        "scan" => {
            scan_ssdp()?;
        }
        _ => print_help(),
    }

    Ok(())
}

fn print_help() {
    println!("usage");
    println!("  cargo run -- reset <IP:PORT>");
    println!("  cargo run -- listen         ");
    println!("  cargo run -- scan           ");
}

fn reset_pico(target: &str) -> io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(Duration::from_secs(2)))?;

    println!("sending bootsel", target);
    socket.send_to(b"BOOTSEL", target)?;

    let mut buf = [0u8; 1024];
    match socket.recv_from(&mut buf) {
        Ok((n, _)) => {
            println!("response", String::from_utf8_lossy(&buf[..n]));
        }
        Err(_) => {
            println!("no response");
        }
    }
    Ok(())
}

fn listen_ssdp() -> io::Result<()> {
    let mcast_group = Ipv4Addr::new(239, 255, 255, 250);
    let port = 1900;
    let any_addr = Ipv4Addr::new(0, 0, 0, 0);

    let socket = UdpSocket::bind((any_addr, port))?;
    socket.join_multicast_v4(&mcast_group, &any_addr)?;

    println!("ssdp listen");

    let mut buf = [0u8; 2048];
    loop {
        let (n, addr) = socket.recv_from(&mut buf)?;
        println!("{}", String::from_utf8_lossy(&buf[..n]));
    }
}

fn scan_ssdp() -> io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(Duration::from_secs(3)))?;
    socket.set_broadcast(true)?;

    let m_search = "M-SEARCH * HTTP/1.1\r\n\
                    HOST: 239.255.255.250:1900\r\n\
                    MAN: \"ssdp:discover\"\r\n\
                    ST: ssdp:all\r\n\
                    MX: 3\r\n\r\n";

    println!("m-search request");
    socket.send_to(m_search.as_bytes(), "239.255.255.250:1900")?;

    let mut buf = [0u8; 2048];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((n, addr)) => {
                let resp = String::from_utf8_lossy(&buf[..n]);
                if resp.contains("Samsung") || resp.contains("Tizen") {
                    println!("\nfound fridge {}", addr);
                    println!("{}", resp);
                }
            }
            Err(_) => {
                println!("scan close");
                break;
            }
        }
    }
    Ok(())
}
