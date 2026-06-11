use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;

/// The built frontend, embedded into the binary at compile time.
/// `build.rs` produces `interface/dist` before this crate compiles.
#[derive(RustEmbed)]
#[folder = "interface/dist"]
struct Assets;

/// Serve embedded static files with an SPA fallback: unknown paths return
/// index.html so client-side routing works on hard refresh.
pub async fn spa(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match Assets::get(path) {
        Some(file) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], file.data).into_response()
        }
        None => match Assets::get("index.html") {
            Some(index) => (
                [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                index.data,
            )
                .into_response(),
            None => (
                StatusCode::NOT_FOUND,
                "frontend not built: run `bun install && bun run build` in interface/",
            )
                .into_response(),
        },
    }
}
