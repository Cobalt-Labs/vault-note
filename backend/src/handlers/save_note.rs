use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    models::{CreateNoteRequest, ErrorResponse, Note, SaveNoteResponse},
    store::NoteStore,
};


fn sanitise(raw: &str) -> String {
    raw.trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn err(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    let msg = msg.into();
    error!(?status, error = %msg, "returning error response");
    (
        status,
        Json(ErrorResponse {
            success: false,
            error: msg,
        }),
    )
}

pub async fn save_note(
    State(store): State<NoteStore>,
    Json(payload): Json<CreateNoteRequest>,
) -> impl IntoResponse {
    let raw_title = match payload.title {
        Some(ref t) => t.as_str(),
        None => {
            warn!("save_note called without a title field");
            return err(
                StatusCode::UNPROCESSABLE_ENTITY,
                "title is required",
            )
            .into_response();
        }
    };

    let title = sanitise(raw_title);
    if title.is_empty() {
        warn!("save_note called with a blank title");
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "title must not be blank",
        )
        .into_response();
    }

    let content = payload
        .content
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_string();

    let id = payload
        .id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let now = Utc::now();

    let note = {
        let mut map = match store.try_write() {
            Ok(guard) => guard,
            Err(_) => {
                error!("NoteStore write lock is poisoned / contended");
                return err(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "note store temporarily unavailable, please retry",
                )
                .into_response();
            }
        };

        let note = if let Some(existing) = map.get(&id) {
            Note {
                id: id.clone(),
                title,
                content,
                created_at: existing.created_at,
                updated_at: now,
            }
        } else {
            Note {
                id: id.clone(),
                title,
                content,
                created_at: now,
                updated_at: now,
            }
        };

        map.insert(id.clone(), note.clone());
        note
    }; // write lock released here

    info!(note_id = %note.id, title = %note.title, "note saved successfully");

    (
        StatusCode::OK,
        Json(SaveNoteResponse {
            success: true,
            message: format!("Note \"{}\" saved.", note.title),
            note,
        }),
    )
        .into_response()
}

pub async fn list_notes(State(store): State<NoteStore>) -> impl IntoResponse {
    let map = match store.try_read() {
        Ok(guard) => guard,
        Err(_) => {
            error!("NoteStore read lock is poisoned / contended");
            return err(
                StatusCode::SERVICE_UNAVAILABLE,
                "note store temporarily unavailable, please retry",
            )
            .into_response();
        }
    };

    let mut notes: Vec<Note> = map.values().cloned().collect();
    notes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    info!(count = notes.len(), "listing notes");
    (StatusCode::OK, Json(notes)).into_response()
}
