use std::io::{self};
use std::net::UdpSocket;
use std::time::Duration;

fn main() -> io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(Duration::from_secs(1)))?;

    let server_addr = "192.168.0.104:1337";
    // let server_addr = "pico2w.local:1337";
    let message = "BOOTSEL";

    socket.send_to(message.as_bytes(), server_addr)?;

    let mut buf = [0u8; 1024];
    match socket.recv_from(&mut buf) {
        Ok((n, addr)) => {
            let echoed = String::from_utf8_lossy(&buf[..n]);
            println!("[{}]: {}", addr, echoed);
        }
        Err(e) if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut => {
            println!("pico no reply.");
        }
        Err(e) => return Err(e),
    }

    Ok(())
}
