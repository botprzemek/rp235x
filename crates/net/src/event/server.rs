//! Server-to-client event types.

use crate::define_event;

define_event! {
    /// Event code sent by the server in a [`crate::ServerPacket`].
    pub enum ServerEvent {
        HandshakeAck = 0x81,
        HeartbeatAck = 0x82,

        GameUpdate = 0xc0,
        GameListUpdate = 0xc1,

        DeviceUpdate = 0xd0,
        DeviceDisconnect = 0xd1,
        DeviceConfigAck = 0xd2,
        DeviceConfigSuccess = 0xd3,
    }
}
