use serde::{Deserialize, Serialize};

/// Sent to the backend when creating/updating a note.
#[derive(Debug, Clone, Serialize)]
pub struct CreateNoteRequest {
    pub id: Option<String>,
    pub title: String,
    pub content: String,
}

/// A fully-hydrated note as returned from the backend.
#[derive(Debug, Clone, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Successful save response.
#[derive(Debug, Clone, Deserialize)]
pub struct SaveNoteResponse {
    pub success: bool,
    pub note: Note,
    pub message: String,
}
