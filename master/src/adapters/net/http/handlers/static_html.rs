use axum::{
    Router,
    body::Body,
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Response},
    routing::get,
};
use hmi::App;
use rust_embed::RustEmbed;
use yew::ServerRenderer;

#[derive(RustEmbed)]
#[folder = "../hmi/dist/"]
struct Assets;

pub struct StaticHandler;

impl StaticHandler {
    pub fn create() -> Router {
        Router::new()
            .route("/", get(Self::render))
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
            format!("{}<body>{}{}", before, rendered_html, after)
        } else {
            rendered_html
        };

        Response::builder()
            .header("content-type", "text/html; charset=utf-8")
            .body(Body::from(html))
            .unwrap()
    }

    async fn fallback(uri: Uri) -> impl IntoResponse {
        let path = uri.path().trim_start_matches('/');
        let path = if path.is_empty() { "index.html" } else { path };

        if let Some(content) = Assets::get(path) {
            let mime = mime_guess::from_path(path).first_or_octet_stream();

            return Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data.into_owned()))
                .unwrap();
        };

        match Assets::get("index.html") {
            Some(content) => Response::builder()
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(Body::from(content.data.into_owned()))
                .unwrap(),
            None => StatusCode::NOT_FOUND.into_response(),
        }
    }
}
