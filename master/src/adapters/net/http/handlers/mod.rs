//! HTTP route handlers for REST API, static SSR assets, and WebSocket connections.

mod api;
mod static_html;
mod websocket;

pub use api::GameHandler;
pub use static_html::StaticHandler;
pub use websocket::WebsocketHandler;
