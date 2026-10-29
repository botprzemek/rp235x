//! REST API route handlers for querying and mutating games and devices.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{self, Router};
use basketball::{Score, Team};

use crate::services::Services;

/// Path parameters for querying a single team's score.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetScoreRequest {
    team: Team,
}

/// JSON response containing a single team's score.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetScoreResponse {
    score: u16,
}

/// JSON request body for adding points to a team.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddScoreRequest {
    team: Team,
    score: Score,
}

/// JSON response containing both home and away team scores.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetScoresResponse {
    home_score: u16,
    away_score: u16,
}

/// Router factory and handlers for `/api` endpoints.
pub struct GameHandler;

impl GameHandler {
    /// Creates the `/api` [`Router`] backed by `services`.
    pub fn create(services: Arc<Services>) -> Router {
        Router::new()
            .route("/devices", routing::get(Self::get_devices))
            .route("/games", routing::get(Self::get_games))
            .route("/scores", routing::get(Self::get_scores))
            .route("/scores", routing::post(Self::add_score))
            .route("/teams/{team}/score", routing::get(Self::get_score))
            .route("/state/resume", routing::post(Self::resume))
            .route("/state/pause", routing::post(Self::pause))
            .with_state(services)
    }

    async fn get_devices(State(services): State<Arc<Services>>) -> impl IntoResponse {
        Json(services.devices().all())
    }

    async fn get_games(State(services): State<Arc<Services>>) -> impl IntoResponse {
        Json(services.game().all())
    }

    async fn get_scores(State(services): State<Arc<Services>>) -> impl IntoResponse {
        let scores = services.game().scores().await;

        Json(GetScoresResponse {
            home_score: scores.home,
            away_score: scores.away,
        })
    }

    async fn get_score(
        State(services): State<Arc<Services>>,
        Path(params): Path<GetScoreRequest>,
    ) -> impl IntoResponse {
        let score = services.game().score(params.team).await;

        Json(GetScoreResponse { score })
    }

    async fn add_score(
        State(services): State<Arc<Services>>,
        Json(body): Json<AddScoreRequest>,
    ) -> impl IntoResponse {
        services.game().add_score(body.team, body.score).await;

        StatusCode::OK
    }

    async fn resume(State(services): State<Arc<Services>>) -> impl IntoResponse {
        services.game().resume().await;

        StatusCode::OK
    }

    async fn pause(State(services): State<Arc<Services>>) -> impl IntoResponse {
        services.game().pause().await;

        StatusCode::OK
    }
}
