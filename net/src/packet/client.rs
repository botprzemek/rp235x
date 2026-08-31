use crate::{ClientEvent, layout};

static PACKET_MAGIC: u8 = 0x42;
static PACKET_VERSION: u8 = 0x01;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ClientPacket {
    pub magic: u8,
    pub event: ClientEvent,
    pub sequence_id: u8,
    pub version: u8,
    pub data: [u8; layout::DATA_SIZE],
    pub crc: u8,
}

impl ClientPacket {
    pub fn new(event: ClientEvent, sequence_id: u8, data: [u8; layout::DATA_SIZE]) -> Self {
        let mut packet = Self {
            magic: PACKET_MAGIC,
            event,
            sequence_id,
            version: PACKET_VERSION,
            data,
            crc: 0,
        };

        packet.crc = packet.calculate_crc();

        packet
    }

    pub fn to_bytes(&self) -> [u8; layout::PACKET_SIZE] {
        let mut bytes = [0u8; layout::PACKET_SIZE];

        bytes[layout::MAGIC] = self.magic;
        bytes[layout::EVENT] = self.event as u8;
        bytes[layout::SEQUENCE_ID] = self.sequence_id;
        bytes[layout::VERSION] = self.version;

        bytes[layout::DATA_START..layout::DATA_END].copy_from_slice(&self.data);
        bytes[layout::CRC] = self.crc;

        bytes
    }

    pub fn from_bytes(buffer: &[u8; layout::PACKET_SIZE]) -> Result<Self, &'static str> {
        if buffer[layout::MAGIC] != PACKET_MAGIC {
            return Err("");
        }

        let packet = Self {
            magic: buffer[layout::MAGIC],
            event: ClientEvent::try_from(buffer[layout::EVENT])?,
            sequence_id: buffer[layout::SEQUENCE_ID],
            version: buffer[layout::VERSION],
            data: {
                let mut data = [0u8; layout::DATA_SIZE];
                data.copy_from_slice(&buffer[layout::DATA_START..layout::DATA_END]);
                data
            },
            crc: buffer[layout::CRC],
        };

        if packet.calculate_crc() != packet.crc {
            return Err("");
        }

        Ok(packet)
    }

    fn calculate_crc(&self) -> u8 {
        let mut crc = self.magic ^ (self.event as u8) ^ self.sequence_id ^ self.version;

        for byte in self.data {
            crc ^= byte;
        }

        crc
    }
}
