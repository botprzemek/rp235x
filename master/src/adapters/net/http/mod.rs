use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

mod handlers;

pub use handlers::StaticHandler;

use crate::{adapters::net::http::handlers::GameHandler, services::Services};

pub async fn run(services: Arc<Services>) {
    let server = Router::new()
        .route("/", get(StaticHandler::render))
        .route("/api/score", post(GameHandler::add_score))
        // .route("/api/start", post(api_match_state_handler))
        // .route("/api/stop", post(api_match_state_handler))
        .fallback(StaticHandler::fallback)
        .with_state(services)
        .layer(CorsLayer::permissive());

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, server).await.unwrap();
}
