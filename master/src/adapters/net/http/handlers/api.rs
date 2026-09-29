use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};

use crate::services::Services;

use basketball::{Score, Team};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetScoreRequest {
    team: Team,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetScoreResponse {
    score: u16,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddScoreRequest {
    team: Team,
    score: Score,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetScoresResponse {
    home_score: u16,
    away_score: u16,
}

pub struct GameHandler;

impl GameHandler {
    pub fn create(services: Arc<Services>) -> Router {
        Router::new()
            .route("/scores", get(Self::get_scores))
            .route("/scores", post(Self::add_score))
            .route("/teams/{team}/score", get(Self::get_score))
            .route("/state/resume", post(Self::resume))
            .route("/state/pause", post(Self::pause))
            .with_state(services)
    }

    async fn get_scores(State(services): State<Arc<Services>>) -> impl IntoResponse {
        let (home_score, away_score) = services.game().get_scores().await;

        Json(GetScoresResponse {
            home_score,
            away_score,
        })
    }

    async fn get_score(
        State(services): State<Arc<Services>>,
        Path(params): Path<GetScoreRequest>,
    ) -> impl IntoResponse {
        let score = services.game().get_score(params.team).await;

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
