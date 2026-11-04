//! Core domain types and traits for HMI application state.

use net::{ClientEvent, ClientPacket, Device, ServerPacket};

/// Application domain events.
#[derive(Debug, Clone)]
pub enum AppAction {
    ConnectWs(String),
    DisconnectWs,
    SendClientEvent(ClientEvent),
    SendPacket(ClientPacket),
    SelectGame(u16),
    SelectDevice(Option<String>),
    CreateGame,
    DisconnectDevice(Device),
    AssignDeviceGame(Device, u16),
    ConfigureWifi {
        target_ip: String,
        serial_number: String,
        device_id: String,
        wifi_ssid: String,
        wifi_password: String,
    },
}

/// Incoming notification from network or external hardware.
#[derive(Debug, Clone)]
pub enum NetworkNotification {
    Connected,
    Disconnected,
    ServerPacketReceived(ServerPacket),
    Log(String),
    Error(String),
}

