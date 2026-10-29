//! Static asset serving and server-side rendering (SSR) for the Yew HMI frontend.

use axum::body::Body;
use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{self, Router};
use hmi::App;
use rust_embed::RustEmbed;
use yew::ServerRenderer;

#[derive(RustEmbed)]
#[folder = "../hmi/dist/"]
struct Assets;

/// Router factory and handlers for serving the HMI web application.
pub struct StaticHandler;

impl StaticHandler {
    /// Creates the static and SSR [`Router`].
    pub fn create() -> Router {
        Router::new()
            .route("/", routing::get(Self::render))
            .fallback(Self::fallback)
    }

    async fn render() -> impl IntoResponse {
        let renderer = ServerRenderer::<App>::new();
        let rendered_html = renderer.render().await;

        let index_html = match Assets::get("index.html") {
            Some(content) => String::from_utf8_lossy(&content.data).into_owned(),
            None => "<html><body></body></html>".to_string(),
        };

        let html = if let Some((before, after)) = index_html.split_once("<body>") {
            format!("{before}<body>{rendered_html}{after}")
        } else {
            rendered_html
        };

        Response::builder()
            .header("content-type", "text/html; charset=utf-8")
            .body(Body::from(html))
            .expect("valid HTTP response with HTML body")
    }

    async fn fallback(uri: Uri) -> impl IntoResponse {
        let path = uri.path().trim_start_matches('/');
        let path = if path.is_empty() { "index.html" } else { path };

        if let Some(content) = Assets::get(path) {
            let mime = mime_guess::from_path(path).first_or_octet_stream();

            return Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data.into_owned()))
                .expect("valid HTTP response for embedded asset");
        }

        match Assets::get("index.html") {
            Some(content) => Response::builder()
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(Body::from(content.data.into_owned()))
                .expect("valid HTTP fallback response for index.html"),
            None => StatusCode::NOT_FOUND.into_response(),
        }
    }
}
