use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerInput {
    None,
    BootSuccess,
    BootFailed,
    CoreSynced,
    WifiConnected,
    WifiFailed,
    ErrorOccurred,
    RecoverySuccess,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum ControllerState {
    Boot,
    CoreSync,
    Networking,
    Running,
    ErrorRecovery,
}

pub struct ControllerStateMachine {
    state: ControllerState,
}

pub static CORE1_READY_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
pub static STATE_SIGNAL: Signal<CriticalSectionRawMutex, ControllerState> = Signal::new();

impl ControllerStateMachine {
    pub const fn new() -> Self {
        Self {
            state: ControllerState::Boot,
        }
    }

    pub fn current_state(&self) -> ControllerState {
        self.state
    }

    pub fn process_event(&mut self, input: ControllerInput) {
        self.state = match (self.state, input) {
            (ControllerState::Boot, ControllerInput::BootSuccess) => ControllerState::CoreSync,

            (ControllerState::Boot, ControllerInput::BootFailed) => ControllerState::ErrorRecovery,

            (ControllerState::CoreSync, ControllerInput::CoreSynced) => ControllerState::Networking,

            (ControllerState::Networking, ControllerInput::WifiConnected) => {
                ControllerState::Running
            }

            (ControllerState::Networking, ControllerInput::WifiFailed) => {
                ControllerState::ErrorRecovery
            }

            (_, ControllerInput::ErrorOccurred) => ControllerState::ErrorRecovery,

            (ControllerState::ErrorRecovery, ControllerInput::RecoverySuccess) => {
                ControllerState::Boot
            }

            _ => self.state,
        };
    }
}
