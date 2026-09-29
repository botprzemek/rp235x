use std::sync::Arc;

use axum::Router;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

mod handlers;

pub use handlers::{GameHandler, StaticHandler, WebsocketHandler};

use crate::services::Services;

pub struct WebDriver {
    listener: TcpListener,
}

impl WebDriver {
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
            .merge(StaticHandler::create())
            .merge(WebsocketHandler::create(services.clone()))
            .nest("/api", GameHandler::create(services.clone()))
            .layer(CorsLayer::permissive());

        axum::serve(self.listener, server).await.unwrap();
    }
}
