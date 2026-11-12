use crate::define_event;

define_event! {

    pub enum ServerEvent {
        HandshakeAck = 0x81,
        HeartbeatAck = 0x82,

        GameUpdate = 0xC0,
        GameListUpdate = 0xC1,

        DeviceUpdate = 0xD0,
        DeviceDisconnect = 0xD1,
        DeviceConfigAck = 0xD2,
        DeviceConfigSuccess = 0xD3,
    }
}
