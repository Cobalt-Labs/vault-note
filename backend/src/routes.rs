use axum::{Router, routing::{delete, get as get_route, post}};
use tower_http::cors::{Any, CorsLayer};

use crate::handlers::save_note::{delete_note, list_notes, save_note};
use crate::store::{new_store, NoteStore};

pub async fn router() -> Router {
    let store: NoteStore = new_store();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/notes", post(save_note).get(get_route(list_notes)))
        .route("/api/notes/:id", delete(delete_note))
        .layer(cors)
        .with_state(store)
}
