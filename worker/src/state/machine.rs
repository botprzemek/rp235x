use basketball::Game;

#[cfg(feature = "rp235x")]
use core::cell::RefCell;
#[cfg(feature = "rp235x")]
use critical_section::Mutex as CsMutex;
#[cfg(feature = "rp235x")]
use defmt::Format;
#[cfg(feature = "rp235x")]
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
#[cfg(feature = "rp235x")]
use embassy_sync::signal::Signal;

use core::sync::atomic::AtomicBool;

pub static IS_CONNECTED_TO_MASTER: AtomicBool = AtomicBool::new(false);

#[cfg(not(feature = "rp235x"))]
use std::sync::RwLock;

#[cfg(feature = "rp235x")]
pub struct SharedGameState {
    state: CsMutex<RefCell<Game>>,
}

#[cfg(feature = "rp235x")]
impl SharedGameState {

    pub const fn new() -> Self {
        Self {
            state: CsMutex::new(RefCell::new(Game::empty())),
        }
    }

    #[inline]
    pub fn get(&self) -> Game {
        critical_section::with(|cs| *self.state.borrow(cs).borrow())
    }

    #[inline]
    pub fn set(&self, game: Game) {
        critical_section::with(|cs| {
            *self.state.borrow(cs).borrow_mut() = game;
        });
    }

    #[inline]
    #[allow(dead_code)]
    pub fn update<F: FnOnce(&mut Game)>(&self, f: F) {
        critical_section::with(|cs| {
            f(&mut self.state.borrow(cs).borrow_mut());
        });
    }
}

#[cfg(feature = "rp235x")]
unsafe impl Sync for SharedGameState {}

#[cfg(not(feature = "rp235x"))]
pub struct SharedGameState {
    state: RwLock<Game>,
}

#[cfg(not(feature = "rp235x"))]
impl SharedGameState {

    pub const fn new() -> Self {
        Self {
            state: RwLock::new(Game::empty()),
        }
    }

    #[inline]
    pub fn get(&self) -> Game {
        *self.state.read().expect("game state read lock")
    }

    #[inline]
    pub fn set(&self, game: Game) {
        *self.state.write().expect("game state write lock") = game;
    }

    #[inline]
    #[allow(dead_code)]
    pub fn update<F: FnOnce(&mut Game)>(&self, f: F) {
        f(&mut self.state.write().expect("game state write lock"));
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "rp235x", derive(Format))]
#[allow(dead_code)]
pub enum Input {
    #[default]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "rp235x", derive(Format))]
pub enum State {
    #[default]
    Boot = 0x01,
    CoreSync = 0x02,
    Networking = 0x03,
    Running = 0x04,
    ErrorRecovery = 0x05,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "rp235x", derive(Format))]
pub struct Machine {
    state: State,
}

#[cfg(feature = "rp235x")]
pub static STATE_SIGNAL: Signal<CriticalSectionRawMutex, State> = Signal::new();

#[cfg(not(feature = "rp235x"))]
pub struct LinuxSignal<T> {
    state: std::sync::Mutex<Option<T>>,
    cvar: std::sync::Condvar,
}

#[cfg(not(feature = "rp235x"))]
impl<T: Copy> LinuxSignal<T> {
    pub const fn new() -> Self {
        Self {
            state: std::sync::Mutex::new(None),
            cvar: std::sync::Condvar::new(),
        }
    }

    pub fn signal(&self, value: T) {
        let mut guard = self.state.lock().expect("signal lock");
        *guard = Some(value);
        self.cvar.notify_all();
    }

    #[allow(dead_code)]
    pub fn wait(&self) -> T {
        let mut guard = self.state.lock().expect("signal lock");
        while guard.is_none() {
            guard = self.cvar.wait(guard).expect("cvar wait");
        }
        guard.take().unwrap()
    }
}

#[cfg(not(feature = "rp235x"))]
pub static STATE_SIGNAL: LinuxSignal<State> = LinuxSignal::new();

pub static GAME_STATE: SharedGameState = SharedGameState::new();

impl Machine {

    #[must_use]
    pub const fn new() -> Self {
        Self { state: State::Boot }
    }

    #[must_use]
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
        #[cfg(feature = "rp235x")]
        defmt::debug!("Input::{}", &input);
        #[cfg(not(feature = "rp235x"))]
        println!("[STATE MACHINE] Input::{:?}", input);

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
