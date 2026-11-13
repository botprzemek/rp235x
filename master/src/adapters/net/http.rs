use std::sync::Arc;

use crate::adapters::net::http::{GameHandler, WebsocketHandler};
use crate::services::Services;

use axum::Router;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

pub struct WebDriver {
    listener: TcpListener,
}

impl WebDriver {
    #[must_use]
    pub fn new(listener: TcpListener) -> Self {
        Self { listener }
    }

    pub fn spawn(self, services: Arc<Services>) {
        tokio::spawn(async move {
            Self::run(self, services).await;
        });
    }

    pub async fn run(self, services: Arc<Services>) {
        let server = Router::new()
            .merge(WebsocketHandler::create(Arc::clone(&services)))
            .layer(CorsLayer::permissive());

        axum::serve(self.listener, server)
            .await
            .expect("HTTP server failed");
    }
}
