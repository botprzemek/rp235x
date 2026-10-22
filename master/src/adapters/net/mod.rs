mod http;
mod udp;

use core::net::SocketAddr;
use core::time::Duration;
use std::sync::Arc;

use basketball::{Discipline, Game, Scorable, Score, Team};
use config::DeviceConfig;
pub use http::WebDriver;
use net::{
    ClientEvent, ClientPacket, Device, DeviceType, NetError, PacketReceiver, PacketSender, Server,
    ServerEvent, ServerHandler, ServerPacket,
};
pub use udp::WorkerDriver;

use crate::services::{Observable, Services};

const TICK_DURATION: Duration = Duration::from_nanos(1_000_000_000 / 60);

pub struct MasterServerHandler {
    services: Arc<Services>,
    sequence_id: u8,
    validated_device: Option<Device>,
    selected_game_id: u16,
}

impl MasterServerHandler {
    pub fn new(services: Arc<Services>) -> Self {
        Self {
            services,
            sequence_id: 0,
            validated_device: None,
            selected_game_id: 0,
        }
    }

    pub fn is_validated(&self) -> bool {
        self.validated_device.is_some()
    }

    pub fn is_web_browser(&self) -> bool {
        matches!(
            self.validated_device,
            Some(device) if device.device_type() == DeviceType::WebBrowser
        )
    }

    pub fn validated_device(&self) -> Option<Device> {
        self.validated_device
    }

    pub fn selected_game_id(&self) -> u16 {
        self.selected_game_id
    }

    pub fn set_selected_game_id(&mut self, game_id: u16) {
        self.selected_game_id = game_id;
        if let Some(mut dev) = self.validated_device {
            dev.set_game_id(game_id);
            self.validated_device = Some(dev);
        }
    }

    pub fn selected_game(&self) -> Option<Game> {
        if self.selected_game_id == 0 {
            return None;
        }
        self.services.game().get_by_id(self.selected_game_id)
    }
}

impl ServerHandler for MasterServerHandler {
    async fn handle_packet(
        &mut self,
        packet: ClientPacket,
        address: Option<SocketAddr>,
    ) -> Option<ServerPacket> {
        self.sequence_id = self.sequence_id.wrapping_add(1);

        match packet.event() {
            ClientEvent::HandshakeRequest | ClientEvent::HandshakeHeartbeat => {
                let mut device = match Device::try_from(packet) {
                    Ok(device) => device,
                    Err(err) => {
                        eprintln!(
                            "WARN: [WS {:?}] Handshake rejected (invalid payload: {})",
                            address, err
                        );
                        return None;
                    }
                };

                let sn = device.get_serial_number().unwrap_or("");
                let existing_assigned = self.services.devices().get_game_id_by_serial(sn);

                let assigned_game_id = match device.device_type() {
                    DeviceType::Worker => existing_assigned.unwrap_or(0),
                    DeviceType::WebBrowser => {
                        if self.selected_game_id != 0 {
                            self.selected_game_id
                        } else {
                            existing_assigned.unwrap_or_else(|| self.services.game().get().id())
                        }
                    }
                };

                self.selected_game_id = assigned_game_id;
                device.set_game_id(assigned_game_id);

                match self.services.devices().validate_and_register(device) {
                    Ok(registered) => {
                        println!(
                            "INFO: [WS {:?}] Validated client {:?} (id={}, sn={}, game_id={})",
                            address,
                            registered.device_type(),
                            registered.get_device_id().unwrap_or("unknown"),
                            registered.get_serial_number().unwrap_or("unknown"),
                            registered.game_id(),
                        );
                        self.validated_device = Some(registered);
                        let game = self
                            .selected_game()
                            .unwrap_or_else(|| Game::with_id(0, Discipline::Fiba5vs5));
                        Some(ServerPacket::new(
                            ServerEvent::HandshakeAck,
                            self.sequence_id,
                            game.to_bytes(),
                        ))
                    }
                    Err(err) => {
                        eprintln!("WARN: [WS {:?}] Device validation failed: {}", address, err);
                        None
                    }
                }
            }
            event => {
                if self.validated_device.is_none() {
                    eprintln!(
                        "WARN: [WS {:?}] Unvalidated client attempted {:?}",
                        address, event
                    );
                    return None;
                }

                let game_id = self.selected_game_id;

                match event {
                    ClientEvent::GameSelect => {
                        if let Ok(requested_game) = Game::try_from(packet)
                            && let Some(found) = self.services.game().get_by_id(requested_game.id())
                        {
                            self.selected_game_id = found.id();
                            if let Some(mut dev) = self.validated_device {
                                dev.set_game_id(self.selected_game_id);
                                self.validated_device = Some(dev);
                                let _ = self.services.devices().validate_and_register(dev);
                            }
                            println!(
                                "INFO: [WS {:?}] Client switched to game id={}",
                                address, self.selected_game_id
                            );
                            return Some(ServerPacket::new(
                                ServerEvent::GameUpdate,
                                self.sequence_id,
                                found.to_bytes(),
                            ));
                        }
                    }
                    ClientEvent::GameCreate => {
                        let discipline = Game::try_from(packet)
                            .map(|g| g.discipline())
                            .unwrap_or(Discipline::Fiba5vs5);
                        let created = self.services.game().create_game(discipline);
                        self.selected_game_id = created.id();
                        if let Some(mut dev) = self.validated_device {
                            dev.set_game_id(self.selected_game_id);
                            self.validated_device = Some(dev);
                            let _ = self.services.devices().validate_and_register(dev);
                        }
                        println!(
                            "INFO: [WS {:?}] Client created and selected game id={}",
                            address, self.selected_game_id
                        );
                        return Some(ServerPacket::new(
                            ServerEvent::GameUpdate,
                            self.sequence_id,
                            created.to_bytes(),
                        ));
                    }
                    ClientEvent::DeviceDisconnect => {
                        if let Ok(target_device) = Device::try_from(packet)
                            && let Ok(target_sn) = target_device.get_serial_number()
                        {
                            println!(
                                "INFO: [WS {:?}] Requested disconnect for device sn={}",
                                address, target_sn
                            );
                            self.services.devices().disconnect_by_serial(target_sn);
                        }
                    }
                    ClientEvent::DeviceAssignGame => {
                        if let Ok(target_device) = Device::try_from(packet)
                            && let Ok(target_sn) = target_device.get_serial_number()
                        {
                            let target_game_id = target_device.game_id();
                            let is_valid = target_game_id == 0
                                || self.services.game().get_by_id(target_game_id).is_some();
                            if is_valid {
                                println!(
                                    "INFO: [WS {:?}] Assigned device sn={} to game id={}",
                                    address, target_sn, target_game_id
                                );
                                self.services
                                    .devices()
                                    .assign_game_by_serial(target_sn, target_game_id);
                            }
                        }
                    }
                    ClientEvent::GameStart | ClientEvent::GameResume => {
                        self.services.game().resume_for(game_id).await;
                    }
                    ClientEvent::GamePause => {
                        self.services.game().pause_for(game_id).await;
                    }
                    ClientEvent::GameEnd => {
                        self.services.game().end_for(game_id).await;
                    }
                    ClientEvent::GameReset => {
                        self.services.game().reset_for(game_id).await;
                    }
                    ClientEvent::GameScoreHomeFT => {
                        self.services
                            .game()
                            .add_score_for(game_id, Team::Home, Score::FreeThrow)
                            .await;
                    }
                    ClientEvent::GameScoreHome2FG => {
                        self.services
                            .game()
                            .add_score_for(game_id, Team::Home, Score::FieldGoal2)
                            .await;
                    }
                    ClientEvent::GameScoreHome3FG => {
                        self.services
                            .game()
                            .add_score_for(game_id, Team::Home, Score::FieldGoal3)
                            .await;
                    }
                    ClientEvent::GameScoreAwayFT => {
                        self.services
                            .game()
                            .add_score_for(game_id, Team::Away, Score::FreeThrow)
                            .await;
                    }
                    ClientEvent::GameScoreAway2FG => {
                        self.services
                            .game()
                            .add_score_for(game_id, Team::Away, Score::FieldGoal2)
                            .await;
                    }
                    ClientEvent::GameScoreAway3FG => {
                        self.services
                            .game()
                            .add_score_for(game_id, Team::Away, Score::FieldGoal3)
                            .await;
                    }
                    ClientEvent::HandshakeRequest | ClientEvent::HandshakeHeartbeat => {
                        unreachable!()
                    }
                }

                None
            }
        }
    }
}

async fn sync_devices_to_browser<T>(
    server: &mut Server<T, MasterServerHandler>,
    known_devices: &mut Vec<Device>,
    current_devices: Vec<Device>,
    sequence_id: &mut u8,
) where
    T: PacketSender + PacketReceiver,
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
            server.send(&packet).await;
        }
    }

    for device in current_devices.iter() {
        let already_sent = known_devices.iter().any(|d| d == device);
        if !already_sent {
            *sequence_id = sequence_id.wrapping_add(1);
            let packet =
                ServerPacket::new(ServerEvent::DeviceUpdate, *sequence_id, device.to_bytes());
            server.send(&packet).await;
        }
    }

    *known_devices = current_devices;
}

async fn sync_games_to_browser<T>(
    server: &mut Server<T, MasterServerHandler>,
    known_games: &mut Vec<Game>,
    current_games: Vec<Game>,
    sequence_id: &mut u8,
) where
    T: PacketSender + PacketReceiver,
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
            server.send(&packet).await;
        }
    }

    *known_games = current_games;
}

pub async fn run<T>(transport: T, services: Arc<Services>)
where
    T: PacketSender + PacketReceiver,
{
    let handler = MasterServerHandler::new(Arc::clone(&services));
    let mut server = Server::new(transport, handler);

    let mut sequence_id: u8 = 0;
    let mut interval = tokio::time::interval(TICK_DURATION);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut game_rx = services.game().watch();
    let mut devices_rx = services.devices().watch();
    let mut disconnect_rx = services.devices().subscribe_disconnect();
    let mut game_assign_rx = services.devices().subscribe_game_assign();
    let mut known_devices: Vec<Device> = Vec::new();
    let mut known_games: Vec<Game> = Vec::new();

    loop {
        tokio::select! {
            biased;

            Ok(target_sn) = disconnect_rx.recv() => {
                if let Some(device) = server.handler().validated_device()
                    && device.get_serial_number() == Ok(target_sn.as_str())
                {
                    println!("INFO: Closing WebSocket connection for disconnected device sn={}", target_sn);
                    break;
                }
            }

            Ok((target_sn, new_game_id)) = game_assign_rx.recv() => {
                if let Some(device) = server.handler().validated_device()
                    && device.get_serial_number() == Ok(target_sn.as_str())
                {
                    server.handler_mut().set_selected_game_id(new_game_id);
                    let game = server
                        .handler()
                        .selected_game()
                        .unwrap_or_else(|| Game::with_id(0, Discipline::Fiba5vs5));
                    sequence_id = sequence_id.wrapping_add(1);
                    let packet = ServerPacket::new(
                        ServerEvent::GameUpdate,
                        sequence_id,
                        game.to_bytes(),
                    );
                    server.send(&packet).await;
                }
            }

            _ = interval.tick() => {
                if server.handler().is_validated()
                    && let Some(game) = server.handler().selected_game()
                {
                    sequence_id = sequence_id.wrapping_add(1);
                    let packet = ServerPacket::new(
                        ServerEvent::GameUpdate,
                        sequence_id,
                        game.to_bytes(),
                    );
                    server.send(&packet).await;
                }
            }

            Ok(()) = game_rx.changed() => {
                interval.reset();
                let current_games = game_rx.borrow_and_update().clone();
                if server.handler().is_validated() {
                    if server.handler().is_web_browser() {
                        sync_games_to_browser(
                            &mut server,
                            &mut known_games,
                            current_games.clone(),
                            &mut sequence_id,
                        )
                        .await;
                    }

                    let selected_id = server.handler().selected_game_id();
                    if selected_id != 0
                        && let Some(game) = current_games.iter().find(|g| g.id() == selected_id)
                    {
                        sequence_id = sequence_id.wrapping_add(1);
                        let packet = ServerPacket::new(
                            ServerEvent::GameUpdate,
                            sequence_id,
                            game.to_bytes(),
                        );
                        server.send(&packet).await;
                    }
                }
            }

            Ok(()) = devices_rx.changed() => {
                let current_devices = devices_rx.borrow_and_update().clone();
                if server.handler().is_web_browser() {
                    sync_devices_to_browser(
                        &mut server,
                        &mut known_devices,
                        current_devices,
                        &mut sequence_id,
                    )
                    .await;
                }
            }

            result = server.poll() => {
                match result {
                    Ok(packet) => {
                        if matches!(
                            packet.event(),
                            ClientEvent::HandshakeRequest | ClientEvent::HandshakeHeartbeat
                        ) && server.handler().is_web_browser()
                        {
                            known_devices.clear();
                            let current_devices = devices_rx.borrow_and_update().clone();
                            sync_devices_to_browser(
                                &mut server,
                                &mut known_devices,
                                current_devices,
                                &mut sequence_id,
                            )
                            .await;

                            known_games.clear();
                            let current_games = game_rx.borrow_and_update().clone();
                            sync_games_to_browser(
                                &mut server,
                                &mut known_games,
                                current_games,
                                &mut sequence_id,
                            )
                            .await;
                        }
                    }
                    Err(NetError::InvalidPacket(err)) => {
                        eprintln!("WARN: Invalid client packet: {}", err);
                    }
                    Err(NetError::Disconnected) => {
                        if let Some(device) = server.handler().validated_device() {
                            println!(
                                "INFO: Client disconnected: {:?} (id={}, sn={})",
                                device.device_type(),
                                device.get_device_id().unwrap_or("unknown"),
                                device.get_serial_number().unwrap_or("unknown")
                            );
                            services.devices().unregister(&device);
                        } else {
                            println!("INFO: Unvalidated client disconnected");
                        }
                        break;
                    }
                }
            }
        }
    }
}
