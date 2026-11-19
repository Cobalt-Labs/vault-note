use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct CreateNoteRequest {
    pub id: Option<String>,
    pub title: String,
    pub content: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct SaveNoteResponse {
    pub success: bool,
    pub note: Note,
    pub message: String,
}
