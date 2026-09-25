use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use net::data::snapshot::{State as SnapshotState, Team};
use rust_embed::RustEmbed;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

use crate::{AppState, database};

use hmi::App;

#[derive(RustEmbed)]
#[folder = "../hmi/dist/"]
struct Assets;

#[derive(serde::Deserialize)]
struct ScoreRequest {
    team: Team,
    points: u16,
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data.into_owned()))
                .unwrap()
        }
        None => match Assets::get("index.html") {
            Some(content) => Response::builder()
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(Body::from(content.data.into_owned()))
                .unwrap(),
            None => StatusCode::NOT_FOUND.into_response(),
        },
    }
}

async fn render_handler() -> impl IntoResponse {
    let renderer = yew::ServerRenderer::<App>::new();
    let rendered_html = renderer.render().await;

    let index_html = match Assets::get("index.html") {
        Some(content) => String::from_utf8_lossy(&content.data).into_owned(),
        None => "<html><body></body></html>".to_string(),
    };

    let html = if let Some((before, after)) = index_html.split_once("<body>") {
        format!("{}<body>{}{}", before, rendered_html, after)
    } else {
        rendered_html
    };

    axum::response::Response::builder()
        .header("content-type", "text/html; charset=utf-8")
        .body(Body::from(html))
        .unwrap()
}

async fn api_score_handler(
    State(state): State<AppState>,
    axum::Json(body): axum::Json<ScoreRequest>,
) -> impl IntoResponse {
    state.state_tx.send_modify(|data| {
        data.make_field_goal(body.team, body.points);
    });
    let current_state = *state.state_tx.borrow();
    database::save(&state.db, &current_state);

    let json = serde_json::to_string(&*state.state_tx.borrow()).unwrap_or_default();
    let _ = state.tx.send(json);
    axum::http::StatusCode::OK
}

async fn api_match_state_handler(
    State(state): State<AppState>,
    uri: axum::http::Uri,
) -> impl IntoResponse {
    let action = uri.path().strip_prefix("/api/").unwrap_or("unknown");
    let new_state = match action {
        "start" => SnapshotState::Running,
        "stop" => SnapshotState::Paused,
        _ => SnapshotState::Idle,
    };

    state.state_tx.send_modify(|data| {
        data.set_state(new_state);
    });
    let current_state = *state.state_tx.borrow();
    database::save(&state.db, &current_state);

    let json = serde_json::to_string(&*state.state_tx.borrow()).unwrap_or_default();
    let _ = state.tx.send(json);
    axum::http::StatusCode::OK
}

pub async fn run(state: AppState) {
    let service = Router::new()
        .route("/", get(render_handler))
        .route("/api/score", post(api_score_handler))
        .route("/api/start", post(api_match_state_handler))
        .route("/api/stop", post(api_match_state_handler))
        .fallback(static_handler)
        .with_state(state)
        .layer(CorsLayer::permissive());

    println!("http://localhost:80/");
    let listener = TcpListener::bind("0.0.0.0:80").await.unwrap();
    axum::serve(listener, service).await.unwrap();
}
