use std::sync::Arc;

use axum::Router;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

mod handlers;

pub use handlers::{GameHandler, StaticHandler, WebsocketHandler};

use crate::services::Services;

pub async fn run(services: Arc<Services>) {
    let server = Router::new()
        .merge(StaticHandler::create())
        .merge(WebsocketHandler::create(services.clone()))
        .nest("/api", GameHandler::create(services.clone()))
        .layer(CorsLayer::permissive());

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, server).await.unwrap();
}
