use crate::define_event;

define_event! {

    pub enum ClientEvent {
        HandshakeRequest = 0x01,
        HandshakeHeartbeat = 0x02,

        GameCreate = 0x10,
        GameDelete = 0x11,

        GameStart = 0x20,
        GameResume = 0x21,
        GamePause = 0x22,
        GameEnd = 0x23,
        GameReset = 0x24,
        GameSelect = 0x25,

        GameScoreHomeFT = 0x30,
        GameScoreHome2FG = 0x31,
        GameScoreHome3FG = 0x32,
        GameScoreAwayFT = 0x40,
        GameScoreAway2FG = 0x41,
        GameScoreAway3FG = 0x42,

        DeviceDisconnect = 0x60,
        DeviceAssignGame = 0x61,
        DeviceConfigSet = 0x62,
    }
}
