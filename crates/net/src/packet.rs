use crate::event::{ClientEvent, ServerEvent};
use crate::layout;

#[cfg(feature = "serde")]
use serde::Serializer;
#[cfg(feature = "serde")]
use serde::ser::SerializeTuple;

const PACKET_MAGIC: u8 = 0x42;
const PACKET_VERSION: u8 = 0x01;

#[must_use]
pub const fn crc16_ccitt(bytes: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    let mut i = 0;

    while i < bytes.len() {
        crc ^= (bytes[i] as u16) << 8;
        let mut bit = 0;

        while bit < 8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }

            bit += 1;
        }

        i += 1;
    }

    crc
}

#[cfg(feature = "serde")]
pub(crate) fn serialize_data<S>(
    data: &[u8; layout::DATA_SIZE],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut tuple = serializer.serialize_tuple(layout::DATA_SIZE)?;

    for byte in data {
        tuple.serialize_element(byte)?;
    }

    tuple.end()
}

pub trait Packet: Sized + Copy + Send + Sync {
    type Event: Copy + Send + Sync;

    fn event(&self) -> Self::Event;

    fn sequence_id(&self) -> u16;

    fn session_token(&self) -> u32;

    fn payload_len(&self) -> u8;

    fn version(&self) -> u8;

    fn data(self) -> [u8; layout::DATA_SIZE];

    fn to_bytes(&self) -> [u8; layout::PACKET_SIZE];

    fn from_bytes(buffer: [u8; layout::PACKET_SIZE]) -> Result<Self, &'static str>;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ClientPacket {
    magic: u8,
    version: u8,
    event: ClientEvent,
    sequence_id: u16,
    session_token: u32,
    payload_len: u8,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_data"))]
    data: [u8; layout::DATA_SIZE],
    crc: u16,
}

impl ClientPacket {
    #[must_use]
    pub fn new(event: ClientEvent, sequence_id: u16, data: [u8; layout::DATA_SIZE]) -> Self {
        Self::with_session(event, sequence_id, 0, layout::DATA_SIZE as u8, data)
    }

    #[must_use]
    pub fn with_session(
        event: ClientEvent,
        sequence_id: u16,
        session_token: u32,
        payload_len: u8,
        data: [u8; layout::DATA_SIZE],
    ) -> Self {
        let clamped_len = payload_len.min(layout::DATA_SIZE as u8);
        let mut packet = Self {
            magic: PACKET_MAGIC,
            version: PACKET_VERSION,
            event,
            sequence_id,
            session_token,
            payload_len: clamped_len,
            data,
            crc: 0,
        };

        packet.crc = packet.calculate_crc();

        packet
    }

    #[must_use]
    pub fn to_bytes(&self) -> [u8; layout::PACKET_SIZE] {
        let mut bytes = [0u8; layout::PACKET_SIZE];

        bytes[layout::MAGIC] = self.magic;
        bytes[layout::VERSION] = self.version;
        bytes[layout::EVENT] = self.event as u8;
        bytes[layout::SEQUENCE_ID_START..layout::SEQUENCE_ID_END]
            .copy_from_slice(&self.sequence_id.to_be_bytes());
        bytes[layout::SESSION_TOKEN_START..layout::SESSION_TOKEN_END]
            .copy_from_slice(&self.session_token.to_be_bytes());
        bytes[layout::PAYLOAD_LEN] = self.payload_len;

        bytes[layout::DATA_START..layout::DATA_END].copy_from_slice(&self.data);
        bytes[layout::CRC_START..layout::CRC_END].copy_from_slice(&self.crc.to_be_bytes());

        bytes
    }

    pub fn from_bytes(buffer: [u8; layout::PACKET_SIZE]) -> Result<Self, &'static str> {
        if buffer[layout::MAGIC] != PACKET_MAGIC {
            return Err("magic");
        }

        let payload_len = buffer[layout::PAYLOAD_LEN];
        if (payload_len as usize) > layout::DATA_SIZE {
            return Err("payload_len");
        }

        let mut seq_bytes = [0u8; 2];
        seq_bytes.copy_from_slice(&buffer[layout::SEQUENCE_ID_START..layout::SEQUENCE_ID_END]);

        let mut token_bytes = [0u8; 4];
        token_bytes
            .copy_from_slice(&buffer[layout::SESSION_TOKEN_START..layout::SESSION_TOKEN_END]);

        let mut crc_bytes = [0u8; 2];
        crc_bytes.copy_from_slice(&buffer[layout::CRC_START..layout::CRC_END]);

        let packet = Self {
            magic: buffer[layout::MAGIC],
            version: buffer[layout::VERSION],
            event: ClientEvent::try_from(buffer[layout::EVENT])?,
            sequence_id: u16::from_be_bytes(seq_bytes),
            session_token: u32::from_be_bytes(token_bytes),
            payload_len,
            data: {
                let mut data = [0u8; layout::DATA_SIZE];
                data.copy_from_slice(&buffer[layout::DATA_START..layout::DATA_END]);
                data
            },
            crc: u16::from_be_bytes(crc_bytes),
        };

        let expected_crc = crc16_ccitt(&buffer[..layout::CRC_START]);
        if expected_crc != packet.crc {
            return Err("crc");
        }

        Ok(packet)
    }

    fn calculate_crc(&self) -> u16 {
        let mut bytes = [0u8; layout::CRC_START];
        bytes[layout::MAGIC] = self.magic;
        bytes[layout::VERSION] = self.version;
        bytes[layout::EVENT] = self.event as u8;
        bytes[layout::SEQUENCE_ID_START..layout::SEQUENCE_ID_END]
            .copy_from_slice(&self.sequence_id.to_be_bytes());
        bytes[layout::SESSION_TOKEN_START..layout::SESSION_TOKEN_END]
            .copy_from_slice(&self.session_token.to_be_bytes());
        bytes[layout::PAYLOAD_LEN] = self.payload_len;
        bytes[layout::DATA_START..layout::DATA_END].copy_from_slice(&self.data);

        crc16_ccitt(&bytes)
    }

    #[must_use]
    pub fn event(&self) -> ClientEvent {
        self.event
    }

    #[must_use]
    pub fn sequence_id(&self) -> u16 {
        self.sequence_id
    }

    #[must_use]
    pub fn session_token(&self) -> u32 {
        self.session_token
    }

    #[must_use]
    pub fn payload_len(&self) -> u8 {
        self.payload_len
    }

    #[must_use]
    pub fn version(&self) -> u8 {
        self.version
    }

    #[must_use]
    pub fn data(self) -> [u8; layout::DATA_SIZE] {
        self.data
    }
}

impl Packet for ClientPacket {
    type Event = ClientEvent;

    fn event(&self) -> Self::Event {
        Self::event(self)
    }

    fn sequence_id(&self) -> u16 {
        Self::sequence_id(self)
    }

    fn session_token(&self) -> u32 {
        Self::session_token(self)
    }

    fn payload_len(&self) -> u8 {
        Self::payload_len(self)
    }

    fn version(&self) -> u8 {
        Self::version(self)
    }

    fn data(self) -> [u8; layout::DATA_SIZE] {
        Self::data(self)
    }

    fn to_bytes(&self) -> [u8; layout::PACKET_SIZE] {
        Self::to_bytes(self)
    }

    fn from_bytes(buffer: [u8; layout::PACKET_SIZE]) -> Result<Self, &'static str> {
        Self::from_bytes(buffer)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ServerPacket {
    magic: u8,
    version: u8,
    event: ServerEvent,
    sequence_id: u16,
    session_token: u32,
    payload_len: u8,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_data"))]
    data: [u8; layout::DATA_SIZE],
    crc: u16,
}

impl ServerPacket {
    #[must_use]
    pub fn new(event: ServerEvent, sequence_id: u16, data: [u8; layout::DATA_SIZE]) -> Self {
        Self::with_session(event, sequence_id, 0, layout::DATA_SIZE as u8, data)
    }

    #[must_use]
    pub fn with_session(
        event: ServerEvent,
        sequence_id: u16,
        session_token: u32,
        payload_len: u8,
        data: [u8; layout::DATA_SIZE],
    ) -> Self {
        let clamped_len = payload_len.min(layout::DATA_SIZE as u8);
        let mut packet = Self {
            magic: PACKET_MAGIC,
            version: PACKET_VERSION,
            event,
            sequence_id,
            session_token,
            payload_len: clamped_len,
            data,
            crc: 0,
        };

        packet.crc = packet.calculate_crc();

        packet
    }

    #[must_use]
    pub fn to_bytes(&self) -> [u8; layout::PACKET_SIZE] {
        let mut bytes = [0u8; layout::PACKET_SIZE];

        bytes[layout::MAGIC] = self.magic;
        bytes[layout::VERSION] = self.version;
        bytes[layout::EVENT] = self.event as u8;
        bytes[layout::SEQUENCE_ID_START..layout::SEQUENCE_ID_END]
            .copy_from_slice(&self.sequence_id.to_be_bytes());
        bytes[layout::SESSION_TOKEN_START..layout::SESSION_TOKEN_END]
            .copy_from_slice(&self.session_token.to_be_bytes());
        bytes[layout::PAYLOAD_LEN] = self.payload_len;

        bytes[layout::DATA_START..layout::DATA_END].copy_from_slice(&self.data);
        bytes[layout::CRC_START..layout::CRC_END].copy_from_slice(&self.crc.to_be_bytes());

        bytes
    }

    pub fn from_bytes(buffer: [u8; layout::PACKET_SIZE]) -> Result<Self, &'static str> {
        if buffer[layout::MAGIC] != PACKET_MAGIC {
            return Err("magic");
        }

        let payload_len = buffer[layout::PAYLOAD_LEN];
        if (payload_len as usize) > layout::DATA_SIZE {
            return Err("payload_len");
        }

        let mut seq_bytes = [0u8; 2];
        seq_bytes.copy_from_slice(&buffer[layout::SEQUENCE_ID_START..layout::SEQUENCE_ID_END]);

        let mut token_bytes = [0u8; 4];
        token_bytes
            .copy_from_slice(&buffer[layout::SESSION_TOKEN_START..layout::SESSION_TOKEN_END]);

        let mut crc_bytes = [0u8; 2];
        crc_bytes.copy_from_slice(&buffer[layout::CRC_START..layout::CRC_END]);

        let packet = Self {
            magic: buffer[layout::MAGIC],
            version: buffer[layout::VERSION],
            event: ServerEvent::try_from(buffer[layout::EVENT])?,
            sequence_id: u16::from_be_bytes(seq_bytes),
            session_token: u32::from_be_bytes(token_bytes),
            payload_len,
            data: {
                let mut data = [0u8; layout::DATA_SIZE];
                data.copy_from_slice(&buffer[layout::DATA_START..layout::DATA_END]);
                data
            },
            crc: u16::from_be_bytes(crc_bytes),
        };

        let expected_crc = crc16_ccitt(&buffer[..layout::CRC_START]);
        if expected_crc != packet.crc {
            return Err("crc");
        }

        Ok(packet)
    }

    fn calculate_crc(&self) -> u16 {
        let mut bytes = [0u8; layout::CRC_START];
        bytes[layout::MAGIC] = self.magic;
        bytes[layout::VERSION] = self.version;
        bytes[layout::EVENT] = self.event as u8;
        bytes[layout::SEQUENCE_ID_START..layout::SEQUENCE_ID_END]
            .copy_from_slice(&self.sequence_id.to_be_bytes());
        bytes[layout::SESSION_TOKEN_START..layout::SESSION_TOKEN_END]
            .copy_from_slice(&self.session_token.to_be_bytes());
        bytes[layout::PAYLOAD_LEN] = self.payload_len;
        bytes[layout::DATA_START..layout::DATA_END].copy_from_slice(&self.data);

        crc16_ccitt(&bytes)
    }

    #[must_use]
    pub fn event(&self) -> ServerEvent {
        self.event
    }

    #[must_use]
    pub fn sequence_id(&self) -> u16 {
        self.sequence_id
    }

    #[must_use]
    pub fn session_token(&self) -> u32 {
        self.session_token
    }

    #[must_use]
    pub fn payload_len(&self) -> u8 {
        self.payload_len
    }

    #[must_use]
    pub fn version(&self) -> u8 {
        self.version
    }

    #[must_use]
    pub fn data(self) -> [u8; layout::DATA_SIZE] {
        self.data
    }
}

impl Packet for ServerPacket {
    type Event = ServerEvent;

    fn event(&self) -> Self::Event {
        Self::event(self)
    }

    fn sequence_id(&self) -> u16 {
        Self::sequence_id(self)
    }

    fn session_token(&self) -> u32 {
        Self::session_token(self)
    }

    fn payload_len(&self) -> u8 {
        Self::payload_len(self)
    }

    fn version(&self) -> u8 {
        Self::version(self)
    }

    fn data(self) -> [u8; layout::DATA_SIZE] {
        Self::data(self)
    }

    fn to_bytes(&self) -> [u8; layout::PACKET_SIZE] {
        Self::to_bytes(self)
    }

    fn from_bytes(buffer: [u8; layout::PACKET_SIZE]) -> Result<Self, &'static str> {
        Self::from_bytes(buffer)
    }
}
