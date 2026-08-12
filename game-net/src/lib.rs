#![no_std]

use game::{DATA_SIZE, Game};

pub const PACKET_SIZE: usize = 27;

static PACKET_MAGIC: u8 = 0x42;
static PACKET_VERSION: u8 = 0x01;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Command {
    Ping = 0x00,
    GetTelemetry = 0x01,
    SetBrightness = 0x02,
    Reboot = 0x0F,

    Start = 0x40,
    Stop = 0x41,
    Reset24 = 0x42,
    Reset14 = 0x43,
    SetTime = 0x44,
    AdjustTime = 0x45,

    Blank = 0x50,
    TestPattern = 0x51,
    SetDisplayMode = 0x52,

    TriggerHorn = 0x60,
    SetAutoHorn = 0x61,

    UpdateScore = 0x70,
    UpdatePeriod = 0x71,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Packet {
    pub magic: u8,
    pub command: Command,
    pub seq_id: u8,
    pub version: u8,
    pub data: [u8; game::DATA_SIZE],
    pub crc: u8,
}

impl TryFrom<u8> for Command {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Command::Ping),
            0x01 => Ok(Command::GetTelemetry),
            0x02 => Ok(Command::SetBrightness),
            0x0F => Ok(Command::Reboot),
            0x40 => Ok(Command::Start),
            0x41 => Ok(Command::Stop),
            0x42 => Ok(Command::Reset24),
            0x43 => Ok(Command::Reset14),
            0x44 => Ok(Command::SetTime),
            0x45 => Ok(Command::AdjustTime),
            0x50 => Ok(Command::Blank),
            0x51 => Ok(Command::TestPattern),
            0x52 => Ok(Command::SetDisplayMode),
            0x60 => Ok(Command::TriggerHorn),
            0x61 => Ok(Command::SetAutoHorn),
            0x70 => Ok(Command::UpdateScore),
            0x71 => Ok(Command::UpdatePeriod),
            _ => Err(""),
        }
    }
}

impl Packet {
    pub fn new(command: Command, seq_id: u8, game: &Game) -> Self {
        let mut p = Self {
            magic: PACKET_MAGIC,
            command,
            seq_id,
            version: PACKET_VERSION,
            data: game.to_bytes(),
            crc: 0,
        };

        p.crc = p.calculate_crc();

        p
    }

    fn calculate_crc(&self) -> u8 {
        let mut crc = self.magic ^ (self.command as u8) ^ self.seq_id ^ self.version;

        for byte in self.data {
            crc ^= byte;
        }

        crc
    }

    pub fn to_bytes(&self) -> [u8; PACKET_SIZE] {
        let mut bytes = [0u8; PACKET_SIZE];

        bytes[0] = self.magic;
        bytes[1] = self.command as u8;
        bytes[2] = self.seq_id;
        bytes[3] = self.version;
        bytes[4..26].copy_from_slice(&self.data);
        bytes[26] = self.crc;

        bytes
    }

    pub fn from_bytes(buffer: &[u8; PACKET_SIZE]) -> Result<Self, &'static str> {
        if buffer[0] != PACKET_MAGIC {
            return Err("");
        }

        let packet = Self {
            magic: buffer[0],
            command: Command::try_from(buffer[1])?,
            seq_id: buffer[2],
            version: buffer[3],
            data: {
                let mut data = [0u8; DATA_SIZE];
                data.copy_from_slice(&buffer[4..26]);
                data
            },
            crc: buffer[26],
        };

        if packet.calculate_crc() != packet.crc {
            return Err("");
        }

        Ok(packet)
    }
}
