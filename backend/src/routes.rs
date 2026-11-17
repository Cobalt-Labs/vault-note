use crate::handlers::save_note::save_note;
use tower_http::cors::{Any, CorsLayer};

use axum::{Router, routing::post};

pub async fn router() -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    Router::new()
        .route("/api/notes", post(save_note))
        .layer(cors)
}
