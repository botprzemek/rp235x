//! Client-to-server event types.

use crate::define_event;

define_event! {
    /// Event code sent by a client in a [`crate::ClientPacket`].
    pub enum ClientEvent {
        HandshakeRequest = 0x01,
        HandshakeHeartbeat = 0x02,

        GameStart = 0x10,
        GameResume = 0x11,
        GamePause = 0x12,
        GameEnd = 0x13,
        GameReset = 0x14,
        GameSelect = 0x15,
        GameCreate = 0x16,

        GameScoreHomeFT = 0x20,
        GameScoreHome2FG = 0x21,
        GameScoreHome3FG = 0x22,

        GameScoreAwayFT = 0x30,
        GameScoreAway2FG = 0x31,
        GameScoreAway3FG = 0x32,

        DeviceDisconnect = 0x40,
        DeviceAssignGame = 0x41,
        DeviceConfigSet = 0x42,
    }
}
