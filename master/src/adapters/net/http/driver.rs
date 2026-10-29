//! Axum [`WebDriver`] serving static assets, REST API routes, and WebSocket upgrades.

use std::sync::Arc;

use axum::Router;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

use crate::adapters::net::http::{GameHandler, StaticHandler, WebsocketHandler};
use crate::services::Services;

/// Axum HTTP server driver.
pub struct WebDriver {
    listener: TcpListener,
}

impl WebDriver {
    /// Creates a new [`WebDriver`] bound to `listener`.
    #[must_use]
    pub fn new(listener: TcpListener) -> Self {
        Self { listener }
    }

    /// Spawns the HTTP server task onto the Tokio runtime.
    pub fn spawn(self, services: Arc<Services>) {
        tokio::spawn(async move {
            Self::run(self, services).await;
        });
    }

    /// Runs the Axum HTTP server until it terminates.
    pub async fn run(self, services: Arc<Services>) {
        let server = Router::new()
            .merge(StaticHandler::create())
            .merge(WebsocketHandler::create(Arc::clone(&services)))
            .nest("/api", GameHandler::create(services))
            .layer(CorsLayer::permissive());

        axum::serve(self.listener, server)
            .await
            .expect("HTTP server failed");
    }
}
