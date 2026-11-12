//! Type-state state machine and storage trait for OTA Wi-Fi / Device Configuration.
//!
//! Separates generic verification, sequence validation, and packet creation
//! from platform-specific persistence (e.g. RP2350 Flash vs Host filesystem).

use crate::data::DeviceConfigPacket;
use crate::event::ServerEvent;
use crate::layout::DATA_SIZE;
use crate::node::NetError;
use crate::packet::ServerPacket;

/// Trait to be implemented by platform adapters (e.g. Flash on RP235x, File on Host).
///
/// The state machine orchestrates the protocol sequence:
/// 1. `DeviceConfigSet` is received from HMI
/// 2. `DeviceConfigAck` is returned
/// 3. [`ConfigStorage::persist_config`] is invoked with verified packet data
/// 4. `DeviceConfigSuccess` is returned
pub trait ConfigStorage {
    /// Adapter-specific storage error.
    type Error;

    /// Persists configuration to non-volatile storage (flash, NVRAM, disk, etc.).
    fn persist_config(&mut self, packet: &DeviceConfigPacket) -> Result<(), Self::Error>;
}

/// Marker states for the OTA configuration process.
pub mod config_state {
    /// Waiting for inbound configuration packets.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Idle;

    /// Inbound packet has been received, acknowledged, and is ready for persistence.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PendingWrite;
}

/// Generic type-state configuration machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigMachine<S> {
    state: S,
}

impl Default for ConfigMachine<config_state::Idle> {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigMachine<config_state::Idle> {
    /// Creates a new configuration state machine in `Idle` state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: config_state::Idle,
        }
    }

    /// Processes an inbound raw packet slice.
    ///
    /// Validates `DeviceConfigPacket`, verifies magic and checksums,
    /// and immediately produces a `DeviceConfigAck` server packet.
    pub fn handle_packet(
        self,
        bytes: &[u8],
    ) -> Result<(ConfigMachine<config_state::PendingWrite>, DeviceConfigPacket, ServerPacket), NetError> {
        let packet = DeviceConfigPacket::from_bytes(bytes)
            .map_err(|_| NetError::InvalidPacket("invalid DeviceConfigPacket format or CRC"))?;

        // Immediately prepare Ack packet
        let ack_packet = ServerPacket::new(ServerEvent::DeviceConfigAck, 0, [0u8; DATA_SIZE]);

        Ok((
            ConfigMachine {
                state: config_state::PendingWrite,
            },
            packet,
            ack_packet,
        ))
    }
}

impl ConfigMachine<config_state::PendingWrite> {
    /// Persists the configuration using the provided [`ConfigStorage`] adapter.
    ///
    /// On success, produces the `DeviceConfigSuccess` server packet and returns to `Idle`.
    pub fn persist_and_complete<S: ConfigStorage>(
        self,
        storage: &mut S,
        packet: &DeviceConfigPacket,
    ) -> Result<(ConfigMachine<config_state::Idle>, ServerPacket), S::Error> {
        storage.persist_config(packet)?;

        let success_packet = ServerPacket::new(ServerEvent::DeviceConfigSuccess, 0, [0u8; DATA_SIZE]);

        Ok((
            ConfigMachine {
                state: config_state::Idle,
            },
            success_packet,
        ))
    }
}

#[cfg(test)]
#[path = "config_machine_tests.rs"]
mod config_machine_tests;
