//! Worker firmware state machine and UDP synchronization handlers.

pub mod handler;
mod machine;

pub use handler::{
    BootHandler, CORE1_READY_SIGNAL, ErrorRecoveryHandler, NetworkingHandler, RunningHandler,
    StateHandler, SyncHandler,
};
pub use machine::{GAME_STATE, Input, Machine, State};
