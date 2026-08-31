pub mod handler;

use defmt::Format;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal,
};
use net::data::snapshot::Snapshot;

#[repr(u8)]
#[derive(Clone, PartialEq, Eq, Format)]
pub enum Input {
    None = 0x00,
    BootSuccess = 0x10,
    BootFailed = 0x11,
    CoreSynced = 0x20,
    WifiConnected = 0x30,
    WifiFailed = 0x31,
    ErrorOccurred = 0x90,
    RecoverySuccess = 0x80,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
pub enum State {
    Boot = 0x01,
    CoreSync = 0x02,
    Networking = 0x03,
    Running = 0x04,
    ErrorRecovery = 0x05,
}

pub struct Machine {
    state: State,
}

pub static STATE_SIGNAL: Signal<CriticalSectionRawMutex, State> = Signal::new();
pub static GAME_CHANNEL: Channel<CriticalSectionRawMutex, Snapshot, 4> = Channel::new();
impl Machine {
    pub const fn new() -> Self {
        Self { state: State::Boot }
    }

    pub fn current_state(&self) -> State {
        self.state
    }

    pub fn transition(&mut self, input: Input) {
        let old_state = self.current_state();
        self.process(input);
        let new_state = self.current_state();

        if old_state != new_state {
            STATE_SIGNAL.signal(new_state);
        }
    }

    fn process(&mut self, input: Input) {
        defmt::debug!("Input::{}", &input);

        self.state = match (self.state, input) {
            (State::Boot, Input::BootSuccess) => State::CoreSync,

            (State::Boot, Input::BootFailed) => State::ErrorRecovery,

            (State::CoreSync, Input::CoreSynced) => State::Networking,

            (State::Networking, Input::WifiConnected) => State::Running,

            (State::Networking, Input::WifiFailed) => State::ErrorRecovery,

            (_, Input::ErrorOccurred) => State::ErrorRecovery,

            (State::ErrorRecovery, Input::RecoverySuccess) => State::Boot,

            _ => self.state,
        };
    }
}
