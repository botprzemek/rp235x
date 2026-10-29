//! Master server application, services, and network/database adapters.

pub mod adapters;
pub mod application;
pub mod services;

pub use application::ApplicationBuilder;
