//! HTTP route handlers for REST API and WebSocket connections.

mod api;
mod websocket;

pub use api::GameHandler;
pub use websocket::WebsocketHandler;
