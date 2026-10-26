use core::time::Duration;
use std::sync::Arc;

use basketball::{Game, Scorable};
use config::DeviceConfig;
use net::layout::PACKET_SIZE;
use net::{
    ClientEvent, Device, NetError, PacketReceiver, PacketSender, Server, ServerEvent, ServerPacket,
    SplitTransport,
};
use tokio::sync::{broadcast, mpsc, watch};

use super::MasterServerHandler;
use crate::services::{GameFrame, Observable, Services};

/// Maximum number of queued control packets (acks, list updates) per connection.
/// A peer that falls this far behind is considered dead and is disconnected.
const OUTBOUND_CAPACITY: usize = 256;
/// Coalescing interval for game/device list updates sent to browsers.
const BROWSER_SYNC_INTERVAL: Duration = Duration::from_millis(100);

/// Connection state shared from the inbound to the outbound half.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Session {
    pub validated: bool,
    pub browser: bool,
    pub game_id: u16,
}

enum Outbound {
    Packet([u8; PACKET_SIZE]),
    /// Resend the full game and device lists (sent after a browser handshake).
    BrowserResync,
}

/// [`PacketSender`] that enqueues packets for the outbound half without ever
/// awaiting socket I/O.
struct QueueSender {
    tx: mpsc::Sender<Outbound>,
    overflowed: bool,
}

impl QueueSender {
    fn push(&mut self, message: Outbound) {
        if self.tx.try_send(message).is_err() {
            self.overflowed = true;
        }
    }
}

impl PacketSender for QueueSender {
    async fn send_packet(&mut self, data: &[u8; PACKET_SIZE]) {
        self.push(Outbound::Packet(*data));
    }
}

pub async fn run<R, W>(receiver: R, sender: W, services: Arc<Services>)
where
    R: PacketReceiver,
    W: PacketSender,
{
    let (out_tx, out_rx) = mpsc::channel(OUTBOUND_CAPACITY);
    let (session_tx, session_rx) = watch::channel(Session::default());

    // Both halves are polled concurrently; a pending `send()` in the outbound
    // half does not prevent the inbound half from making progress. The
    // connection ends as soon as the inbound half finishes.
    tokio::select! {
        _ = read_loop(receiver, out_tx, session_tx, Arc::clone(&services)) => {}
        _ = write_loop(sender, out_rx, session_rx, services) => {}
    }
}

async fn read_loop<R>(
    receiver: R,
    out_tx: mpsc::Sender<Outbound>,
    session_tx: watch::Sender<Session>,
    services: Arc<Services>,
) where
    R: PacketReceiver,
{
    let handler = MasterServerHandler::new(Arc::clone(&services));
    let queue = QueueSender {
        tx: out_tx,
        overflowed: false,
    };
    let mut server = Server::new(SplitTransport::new(queue, receiver), handler);

    let mut disconnect_rx = services.devices().subscribe_disconnect();
    let mut game_assign_rx = services.devices().subscribe_game_assign();

    loop {
        tokio::select! {
            res = disconnect_rx.recv() => {
                if let Ok(target_sn) = res
                    && server.handler().is_device(&target_sn)
                {
                    println!("INFO: Closing WebSocket connection for disconnected device sn={}", target_sn);
                    return;
                }
            }

            // Assignment is event driven: the device registry is only queried
            // when an assignment for this device actually happens.
            res = game_assign_rx.recv() => {
                match res {
                    Ok((target_sn, _)) if server.handler().is_device(&target_sn) => {
                        server.handler_mut().sync_assignment();
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        server.handler_mut().sync_assignment();
                    }
                    _ => {}
                }
            }

            result = server.poll() => {
                match result {
                    Ok(packet) => {
                        if server.handler().is_web_browser()
                            && matches!(
                                packet.event(),
                                ClientEvent::HandshakeRequest | ClientEvent::HandshakeHeartbeat
                            )
                        {
                            // Publish the session before the resync request so
                            // the outbound half already sees a browser session.
                            session_tx.send_if_modified(|s| replace(s, server.handler().session()));
                            server.transport_mut().sender_mut().push(Outbound::BrowserResync);
                        }
                    }
                    Err(NetError::InvalidPacket(err)) => {
                        eprintln!("WARN: Invalid client packet: {}", err);
                    }
                    Err(NetError::Disconnected) => {
                        if let Some(device) = server.handler().validated_device() {
                            if server.handler().is_web_browser() {
                                println!(
                                    "INFO: Client disconnected: {:?} (id={}, sn={})",
                                    device.device_type(),
                                    device.get_device_id().unwrap_or("unknown"),
                                    device.get_serial_number().unwrap_or("unknown")
                                );
                            }
                            services.devices().unregister(&device);
                        } else {
                            println!("INFO: Unvalidated client disconnected");
                        }
                        return;
                    }
                }

                if server.transport().sender().overflowed {
                    eprintln!("WARN: Outbound queue overflow, dropping slow client");
                    if let Some(device) = server.handler().validated_device() {
                        services.devices().unregister(&device);
                    }
                    return;
                }
            }
        }

        session_tx.send_if_modified(|s| replace(s, server.handler().session()));
    }
}

fn replace(current: &mut Session, next: Session) -> bool {
    if *current == next {
        return false;
    }
    *current = next;
    true
}

async fn write_loop<W>(
    mut sender: W,
    mut out_rx: mpsc::Receiver<Outbound>,
    mut session_rx: watch::Receiver<Session>,
    services: Arc<Services>,
) where
    W: PacketSender,
{
    let mut sequence_id: u8 = 0;
    let mut session = Session::default();
    let mut frame_rx: Option<watch::Receiver<Option<GameFrame>>> = None;

    let mut game_rx: Option<watch::Receiver<Vec<Game>>> = None;
    let mut devices_rx: Option<watch::Receiver<Vec<Device>>> = None;
    let mut known_devices: Vec<Device> = Vec::new();
    let mut known_games: Vec<Game> = Vec::new();
    let mut devices_dirty = false;
    let mut games_dirty = false;

    let mut browser_sync_interval = tokio::time::interval(BROWSER_SYNC_INTERVAL);
    browser_sync_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            message = out_rx.recv() => {
                match message {
                    Some(Outbound::Packet(bytes)) => sender.send_packet(&bytes).await,
                    Some(Outbound::BrowserResync) => {
                        if !session.browser {
                            continue;
                        }
                        let games = game_rx.get_or_insert_with(|| services.game().watch());
                        let devices = devices_rx.get_or_insert_with(|| services.devices().watch());

                        known_devices.clear();
                        let current_devices = devices.borrow_and_update().clone();
                        sync_devices_to_browser(&mut sender, &mut known_devices, current_devices, &mut sequence_id).await;

                        known_games.clear();
                        let current_games = games.borrow_and_update().clone();
                        sync_games_to_browser(&mut sender, &mut known_games, current_games, &mut sequence_id).await;

                        devices_dirty = false;
                        games_dirty = false;
                    }
                    None => return,
                }
            }

            res = session_rx.changed() => {
                if res.is_err() {
                    return;
                }
                let next = *session_rx.borrow_and_update();
                let stream_game = next.validated && next.game_id != 0;
                if !stream_game {
                    frame_rx = None;
                } else if frame_rx.is_none() || next.game_id != session.game_id {
                    frame_rx = Some(services.game().subscribe_frames(next.game_id));
                }
                session = next;
            }

            res = changed(&mut frame_rx) => {
                match res {
                    Ok(()) => {
                        let frame = frame_rx.as_mut().and_then(|rx| rx.borrow_and_update().clone());
                        if let Some(frame) = frame {
                            sender.send_packet(&frame).await;
                        }
                    }
                    Err(_) => frame_rx = None,
                }
            }

            res = changed(&mut game_rx) => {
                if res.is_ok() {
                    games_dirty = true;
                }
            }

            res = changed(&mut devices_rx) => {
                if res.is_ok() {
                    devices_dirty = true;
                }
            }

            _ = browser_sync_interval.tick(), if session.browser => {
                if games_dirty && let Some(rx) = &mut game_rx {
                    games_dirty = false;
                    let current_games = rx.borrow_and_update().clone();
                    sync_games_to_browser(&mut sender, &mut known_games, current_games, &mut sequence_id).await;
                }

                if devices_dirty && let Some(rx) = &mut devices_rx {
                    devices_dirty = false;
                    let current_devices = rx.borrow_and_update().clone();
                    sync_devices_to_browser(&mut sender, &mut known_devices, current_devices, &mut sequence_id).await;
                }
            }
        }
    }
}

/// Resolves when the optional watch receiver observes a change; pends forever
/// when there is no receiver.
async fn changed<T>(rx: &mut Option<watch::Receiver<T>>) -> Result<(), watch::error::RecvError> {
    match rx {
        Some(rx) => rx.changed().await,
        None => std::future::pending().await,
    }
}

async fn sync_devices_to_browser<W>(
    sender: &mut W,
    known_devices: &mut Vec<Device>,
    current_devices: Vec<Device>,
    sequence_id: &mut u8,
) where
    W: PacketSender,
{
    for previous in known_devices.iter() {
        let still_present = current_devices
            .iter()
            .any(|d| d.get_serial_number() == previous.get_serial_number());
        if !still_present {
            *sequence_id = sequence_id.wrapping_add(1);
            let packet = ServerPacket::new(
                ServerEvent::DeviceDisconnect,
                *sequence_id,
                previous.to_bytes(),
            );
            sender.send_packet(&packet.to_bytes()).await;
        }
    }

    for device in current_devices.iter() {
        let already_sent = known_devices.iter().any(|d| d == device);
        if !already_sent {
            *sequence_id = sequence_id.wrapping_add(1);
            let packet =
                ServerPacket::new(ServerEvent::DeviceUpdate, *sequence_id, device.to_bytes());
            sender.send_packet(&packet.to_bytes()).await;
        }
    }

    *known_devices = current_devices;
}

async fn sync_games_to_browser<W>(
    sender: &mut W,
    known_games: &mut Vec<Game>,
    current_games: Vec<Game>,
    sequence_id: &mut u8,
) where
    W: PacketSender,
{
    for game in current_games.iter() {
        let unchanged = known_games.iter().any(|g| {
            g.id() == game.id()
                && g.get_scores() == game.get_scores()
                && g.get_state() == game.get_state()
                && g.get_quarter() == game.get_quarter()
        });
        if !unchanged {
            *sequence_id = sequence_id.wrapping_add(1);
            let packet =
                ServerPacket::new(ServerEvent::GameListUpdate, *sequence_id, game.to_bytes());
            sender.send_packet(&packet.to_bytes()).await;
        }
    }

    *known_games = current_games;
}
