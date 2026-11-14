use crate::handlers::save_note::save_note;

use axum::{Router, routing::post};

pub async fn router() -> Router {
    Router::new().route("/api/notes", post(save_note))
}
