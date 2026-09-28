use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{StatusCode, Uri},
    response::IntoResponse,
};

use crate::services::{GameService, Observable, Services};

use basketball::{Scorable, Score, Team};

#[derive(serde::Deserialize)]
pub struct ScoreRequest {
    team: Team,
    points: Score,
}

pub struct GameHandler;

impl GameHandler {
    pub async fn add_score(
        State(services): State<Arc<Services>>,
        Json(body): Json<ScoreRequest>,
    ) -> impl IntoResponse {
        let mut game = services.game().get_game();

        game.score(body.team, body.points);

        services.game().update_game(game);

        StatusCode::OK
    }

    // pub async fn post_state(State(state): State<Services>, uri: Uri) -> impl IntoResponse {
    //     let action = uri.path().strip_prefix("/api/").unwrap_or("unknown");
    //     let new_state = match action {
    //         "start" => SnapshotState::Running,
    //         "stop" => SnapshotState::Paused,
    //         _ => SnapshotState::Idle,
    //     };

    //     state.state_tx.send_modify(|data| {
    //         data.set_state(new_state);
    //     });
    //     let current_state = *state.state_tx.borrow();
    //     database::save(&state.db, &current_state);

    //     let json = serde_json::to_string(&*state.state_tx.borrow()).unwrap_or_default();
    //     let _ = state.tx.send(json);
    //     axum::http::StatusCode::OK
    // }
}
