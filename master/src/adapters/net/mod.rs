mod connection;
mod http;
mod udp;

use core::net::SocketAddr;
use std::sync::Arc;

use basketball::{Discipline, Game, Score, Team};
use config::DeviceConfig;
pub use connection::{Session, run};
pub use http::WebDriver;
use net::{
    ClientEvent, ClientPacket, Device, DeviceType, ServerEvent, ServerHandler, ServerPacket,
};
pub use udp::WorkerDriver;

use crate::services::Services;

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

    #[allow(dead_code)]
    pub fn selected_game_id(&self) -> u16 {
        self.selected_game_id
    }

    /// Snapshot of the connection state consumed by the outbound task.
    pub fn session(&self) -> Session {
        Session {
            validated: self.is_validated(),
            browser: self.is_web_browser(),
            game_id: self.selected_game_id,
        }
    }

    /// Whether this connection belongs to the device with the given serial number.
    pub fn is_device(&self, serial_number: &str) -> bool {
        self.validated_device
            .is_some_and(|dev| dev.get_serial_number() == Ok(serial_number))
    }

    pub fn set_selected_game_id(&mut self, game_id: u16) {
        self.selected_game_id = game_id;
        if let Some(mut dev) = self.validated_device {
            dev.set_game_id(game_id);
            self.validated_device = Some(dev);
        }
    }

    pub fn sync_assignment(&mut self) {
        if let Some(dev) = self.validated_device
            && let Ok(sn) = dev.get_serial_number()
            && let Some(assigned) = self.services.devices().get_game_id_by_serial(sn)
            && assigned != 0
            && assigned != self.selected_game_id
        {
            self.set_selected_game_id(assigned);
        }
    }

    pub fn selected_game(&mut self) -> Option<Game> {
        if self.selected_game_id == 0 {
            self.sync_assignment();
            if self.selected_game_id == 0 {
                return None;
            }
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
            ClientEvent::HandshakeHeartbeat if self.validated_device.is_some() => {
                let game = self
                    .selected_game()
                    .unwrap_or_else(|| Game::with_id(0, Discipline::Fiba5vs5));
                Some(ServerPacket::new(
                    ServerEvent::HandshakeAck,
                    self.sequence_id,
                    game.to_bytes(),
                ))
            }
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
                        if registered.device_type() == DeviceType::WebBrowser {
                            println!(
                                "INFO: [WS {:?}] Validated client {:?} (id={}, sn={}, game_id={})",
                                address,
                                registered.device_type(),
                                registered.get_device_id().unwrap_or("unknown"),
                                registered.get_serial_number().unwrap_or("unknown"),
                                registered.game_id(),
                            );
                        }
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
                                self.services
                                    .devices()
                                    .assign_game_by_serial(target_sn, target_game_id);
                            }
                        }
                    }
                    ClientEvent::GameStart | ClientEvent::GameResume => {
                        let target_id = Game::try_from(packet)
                            .map(|g| g.id())
                            .ok()
                            .filter(|&id| id != 0)
                            .unwrap_or(game_id);
                        self.services.game().resume_for(target_id).await;
                    }
                    ClientEvent::GamePause => {
                        let target_id = Game::try_from(packet)
                            .map(|g| g.id())
                            .ok()
                            .filter(|&id| id != 0)
                            .unwrap_or(game_id);
                        self.services.game().pause_for(target_id).await;
                    }
                    ClientEvent::GameEnd => {
                        let target_id = Game::try_from(packet)
                            .map(|g| g.id())
                            .ok()
                            .filter(|&id| id != 0)
                            .unwrap_or(game_id);
                        self.services.game().end_for(target_id).await;
                    }
                    ClientEvent::GameReset => {
                        let target_id = Game::try_from(packet)
                            .map(|g| g.id())
                            .ok()
                            .filter(|&id| id != 0)
                            .unwrap_or(game_id);
                        self.services.game().reset_for(target_id).await;
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
