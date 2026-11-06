//! Worker lifecycle state machine and shared game state primitives.

use core::cell::RefCell;

use basketball::Game;
use critical_section::Mutex as CsMutex;
use defmt::Format;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;

/// Lock-free, non-blocking synchronized Game state container between Core 0 and Core 1.
pub struct SharedGameState {
    state: CsMutex<RefCell<Game>>,
}

impl SharedGameState {
    /// Creates a new [`SharedGameState`] containing an empty initial game.
    pub const fn new() -> Self {
        Self {
            state: CsMutex::new(RefCell::new(Game::empty())),
        }
    }

    /// Instant, non-blocking snapshot read for Core 1 (zero awaits, zero display jitter).
    #[inline]
    pub fn get(&self) -> Game {
        critical_section::with(|cs| *self.state.borrow(cs).borrow())
    }

    /// Instant, non-blocking atomic update for Core 0.
    #[inline]
    pub fn set(&self, game: Game) {
        critical_section::with(|cs| {
            *self.state.borrow(cs).borrow_mut() = game;
        });
    }

    /// Mutates the game state safely under a fast critical section.
    #[inline]
    #[allow(dead_code)]
    pub fn update<F: FnOnce(&mut Game)>(&self, f: F) {
        critical_section::with(|cs| {
            f(&mut *self.state.borrow(cs).borrow_mut());
        });
    }
}

unsafe impl Sync for SharedGameState {}

/// State transition events for [`Machine`].
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Format)]
#[allow(dead_code)]
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

/// Lifecycle states of the RP235x worker.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
pub enum State {
    Boot = 0x01,
    CoreSync = 0x02,
    Networking = 0x03,
    Running = 0x04,
    ErrorRecovery = 0x05,
}

/// Finite state machine tracking the worker's boot and network lifecycle.
pub struct Machine {
    state: State,
}

/// Signal broadcasting worker [`State`] transitions.
pub static STATE_SIGNAL: Signal<CriticalSectionRawMutex, State> = Signal::new();

/// Shared [`Game`] state updated by Core 0 UDP networking and rendered by Core 1.
pub static GAME_STATE: SharedGameState = SharedGameState::new();

impl Default for Machine {
    fn default() -> Self {
        Self::new()
    }
}

impl Machine {
    /// Creates a new [`Machine`] in [`State::Boot`].
    #[must_use]
    pub const fn new() -> Self {
        Self { state: State::Boot }
    }

    /// Returns the current [`State`] of the machine.
    #[must_use]
    pub fn current_state(&self) -> State {
        self.state
    }

    /// Applies `input` to the state machine and signals [`STATE_SIGNAL`] if the state changed.
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

