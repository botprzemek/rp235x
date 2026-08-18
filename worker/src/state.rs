use defmt::Format;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    None,
    BootSuccess,
    BootFailed,
    CoreSynced,
    WifiConnected,
    WifiFailed,
    ErrorOccurred,
    RecoverySuccess,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
pub enum State {
    Boot,
    CoreSync,
    Networking,
    Running,
    ErrorRecovery,
}

pub struct Machine {
    state: State,
}

pub static STATE_SIGNAL: Signal<CriticalSectionRawMutex, State> = Signal::new();

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
