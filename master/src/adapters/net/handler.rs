//! [`ServerHandler`] implementation for the master server.

use core::net::SocketAddr;
use std::sync::Arc;

use basketball::{Discipline, Game, Score, Team};
use config::DeviceConfig;
use net::{
    ClientEvent, ClientPacket, Device, DeviceType, ServerEvent, ServerHandler, ServerPacket,
};

use crate::adapters::net::Session;
use crate::services::Services;

/// Stateful per-connection packet handler for the master server.
pub struct MasterServerHandler {
    services: Arc<Services>,
    sequence_id: u8,
    validated_device: Option<Device>,
    selected_game_id: u16,
}

impl MasterServerHandler {
    /// Creates a new [`MasterServerHandler`] backed by `services`.
    #[must_use]
    pub fn new(services: Arc<Services>) -> Self {
        Self {
            services,
            sequence_id: 0,
            validated_device: None,
            selected_game_id: 0,
        }
    }

    /// Returns `true` if this connection has completed a valid device handshake.
    #[must_use]
    pub fn is_validated(&self) -> bool {
        self.validated_device.is_some()
    }

    /// Returns `true` if the validated device is a [`DeviceType::WebBrowser`].
    #[must_use]
    pub fn is_web_browser(&self) -> bool {
        matches!(
            self.validated_device,
            Some(device) if device.device_type() == DeviceType::WebBrowser
        )
    }

    /// Returns the validated [`Device`] for this connection, if any.
    #[must_use]
    pub fn validated_device(&self) -> Option<Device> {
        self.validated_device
    }

    /// Returns the currently selected game ID for this connection.
    #[must_use]
    #[allow(dead_code)]
    pub fn selected_game_id(&self) -> u16 {
        self.selected_game_id
    }

    /// Snapshot of the connection state consumed by the outbound task.
    #[must_use]
    pub fn session(&self) -> Session {
        Session {
            validated: self.is_validated(),
            browser: self.is_web_browser(),
            game_id: self.selected_game_id,
        }
    }

    /// Whether this connection belongs to the device with the given serial number.
    #[must_use]
    pub fn is_device(&self, serial_number: &str) -> bool {
        self.validated_device
            .is_some_and(|device| device.serial_number() == Ok(serial_number))
    }

    /// Updates the selected game ID and synchronizes it onto the validated device record.
    pub fn set_selected_game_id(&mut self, game_id: u16) {
        self.selected_game_id = game_id;
        if let Some(mut device) = self.validated_device {
            device.set_game_id(game_id);
            self.validated_device = Some(device);
        }
    }

    /// Synchronizes the connection's selected game ID with the device registry.
    pub fn sync_assignment(&mut self) {
        if let Some(device) = self.validated_device
            && let Ok(serial_number) = device.serial_number()
            && let Some(assigned) = self.services.devices().game_id_by_serial(serial_number)
            && assigned != 0
            && assigned != self.selected_game_id
        {
            self.set_selected_game_id(assigned);
        }
    }

    /// Returns the [`Game`] currently selected by this connection, if any.
    pub fn selected_game(&mut self) -> Option<Game> {
        if self.selected_game_id == 0 {
            self.sync_assignment();
            if self.selected_game_id == 0 {
                return None;
            }
        }
        self.services.game().by_id(self.selected_game_id)
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
                    .unwrap_or_else(|| Game::new_with_id(0, Discipline::Fiba5vs5));
                Some(ServerPacket::new(
                    ServerEvent::HandshakeAck,
                    self.sequence_id,
                    game.to_bytes(),
                ))
            }
            ClientEvent::HandshakeRequest | ClientEvent::HandshakeHeartbeat => {
                let mut device = match Device::try_from(packet) {
                    Ok(device) => device,
                    Err(error) => {
                        eprintln!(
                            "WARN: [WS {address:?}] Handshake rejected (invalid payload: {error})"
                        );
                        return None;
                    }
                };

                let serial_number = device.serial_number().unwrap_or("");
                let existing_assigned = self.services.devices().game_id_by_serial(serial_number);

                let assigned_game_id = match device.device_type() {
                    DeviceType::WebBrowser => {
                        if self.selected_game_id != 0 {
                            self.selected_game_id
                        } else {
                            existing_assigned
                                .unwrap_or_else(|| self.services.game().default_game().id())
                        }
                    }
                    DeviceType::Worker | _ => existing_assigned.unwrap_or(0),
                };

                self.selected_game_id = assigned_game_id;
                device.set_game_id(assigned_game_id);

                match self.services.devices().validate_and_register(device) {
                    Ok(registered) => {
                        if registered.device_type() == DeviceType::WebBrowser {
                            println!(
                                "INFO: [WS {address:?}] Validated client {:?} (id={}, sn={}, game_id={})",
                                registered.device_type(),
                                registered.device_id().unwrap_or("unknown"),
                                registered.serial_number().unwrap_or("unknown"),
                                registered.game_id(),
                            );
                        }
                        self.validated_device = Some(registered);
                        let game = self
                            .selected_game()
                            .unwrap_or_else(|| Game::new_with_id(0, Discipline::Fiba5vs5));
                        Some(ServerPacket::new(
                            ServerEvent::HandshakeAck,
                            self.sequence_id,
                            game.to_bytes(),
                        ))
                    }
                    Err(error) => {
                        eprintln!("WARN: [WS {address:?}] Device validation failed: {error}");
                        None
                    }
                }
            }
            event => {
                if self.validated_device.is_none() {
                    eprintln!("WARN: [WS {address:?}] Unvalidated client attempted {event:?}");
                    return None;
                }

                let game_id = self.selected_game_id;

                match event {
                    ClientEvent::GameSelect => {
                        if let Ok(requested_game) = Game::try_from(packet)
                            && let Some(found) = self.services.game().by_id(requested_game.id())
                        {
                            self.selected_game_id = found.id();
                            if let Some(mut device) = self.validated_device {
                                device.set_game_id(self.selected_game_id);
                                self.validated_device = Some(device);
                                let _ = self.services.devices().validate_and_register(device);
                            }
                            println!(
                                "INFO: [WS {address:?}] Client switched to game id={}",
                                self.selected_game_id
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
                            .map(|game| game.discipline())
                            .unwrap_or(Discipline::Fiba5vs5);
                        let created = self.services.game().create_game(discipline);
                        self.selected_game_id = created.id();
                        if let Some(mut device) = self.validated_device {
                            device.set_game_id(self.selected_game_id);
                            self.validated_device = Some(device);
                            let _ = self.services.devices().validate_and_register(device);
                        }
                        println!(
                            "INFO: [WS {address:?}] Client created and selected game id={}",
                            self.selected_game_id
                        );
                        return Some(ServerPacket::new(
                            ServerEvent::GameUpdate,
                            self.sequence_id,
                            created.to_bytes(),
                        ));
                    }
                    ClientEvent::DeviceDisconnect => {
                        if let Ok(target_device) = Device::try_from(packet)
                            && let Ok(target_serial_number) = target_device.serial_number()
                        {
                            println!(
                                "INFO: [WS {address:?}] Requested disconnect for device sn={target_serial_number}"
                            );
                            self.services
                                .devices()
                                .disconnect_by_serial(target_serial_number);
                        }
                    }
                    ClientEvent::DeviceAssignGame => {
                        if let Ok(target_device) = Device::try_from(packet)
                            && let Ok(target_serial_number) = target_device.serial_number()
                        {
                            let target_game_id = target_device.game_id();
                            let is_valid = target_game_id == 0
                                || self.services.game().by_id(target_game_id).is_some();
                            if is_valid {
                                self.services
                                    .devices()
                                    .assign_game_by_serial(target_serial_number, target_game_id);
                            }
                        }
                    }
                    ClientEvent::GameStart | ClientEvent::GameResume => {
                        let target_id = Game::try_from(packet)
                            .map(|game| game.id())
                            .ok()
                            .filter(|&id| id != 0)
                            .unwrap_or(game_id);
                        self.services.game().resume_for(target_id).await;
                    }
                    ClientEvent::GamePause => {
                        let target_id = Game::try_from(packet)
                            .map(|game| game.id())
                            .ok()
                            .filter(|&id| id != 0)
                            .unwrap_or(game_id);
                        self.services.game().pause_for(target_id).await;
                    }
                    ClientEvent::GameEnd => {
                        let target_id = Game::try_from(packet)
                            .map(|game| game.id())
                            .ok()
                            .filter(|&id| id != 0)
                            .unwrap_or(game_id);
                        self.services.game().end_for(target_id).await;
                    }
                    ClientEvent::GameReset => {
                        let target_id = Game::try_from(packet)
                            .map(|game| game.id())
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
                    _ => {}
                }

                None
            }
        }
    }
}
