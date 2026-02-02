//! Static file serving using rust-embed

use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
};
use rust_embed::RustEmbed;

/// Embed the client directory at compile time
#[derive(RustEmbed)]
#[folder = "../../client"]
struct ClientAssets;

/// Serve static files from the embedded client directory
pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    
    // Default to index.html for root path
    let path = if path.is_empty() { "index.html" } else { path };
    
    match ClientAssets::get(path) {
        Some(content) => {
            let mime_type = mime_guess::from_path(path).first_or_text_plain();
            
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime_type.as_ref())
                .body(Body::from(content.data))
                .unwrap()
        }
        None => {
            // Try to serve index.html for client-side routing
            if let Some(content) = ClientAssets::get("index.html") {
                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "text/html")
                    .body(Body::from(content.data))
                    .unwrap()
            } else {
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("Not Found"))
                    .unwrap()
            }
        }
    }
}

/// Serve the index.html file
pub async fn index_handler() -> impl IntoResponse {
    match ClientAssets::get("index.html") {
        Some(content) => {
            let html = String::from_utf8_lossy(&content.data);
            Html(html.to_string())
        }
        None => {
            Html("<h1>Canopy Server</h1><p>Client files not found.</p>".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assets_exist() {
        // Test that embedded assets exist
        assert!(ClientAssets::get("index.html").is_some());
        assert!(ClientAssets::get("graph.js").is_some());
        assert!(ClientAssets::get("protocol.js").is_some());
    }

    #[test]
    fn test_nonexistent_asset() {
        assert!(ClientAssets::get("nonexistent.file").is_none());
    }
}
