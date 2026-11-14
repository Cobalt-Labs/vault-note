use axum::Json;

use crate::models::Note;

pub async fn save_note(Json(note): Json<Note>) -> String {
    format!("Note saved successfully!! {}", note.title)
}
