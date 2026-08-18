use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

// use crate::shared::state::SystemState;

use game_net::Packet;

pub static GAME_CHANNEL: Channel<CriticalSectionRawMutex, Packet, 10> = Channel::new();

// pub static SYSTEM_STATE: Channel<CriticalSectionRawMutex, SystemState, 2> = Channel::new();
