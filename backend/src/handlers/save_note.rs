use axum::{
    Json,
    extract::{Path, State},
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

// ── helpers ─────────────────────────────────────────────────────────────────

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

// ── POST /api/notes ──────────────────────────────────────────────────────────

pub async fn save_note(
    State(store): State<NoteStore>,
    Json(payload): Json<CreateNoteRequest>,
) -> impl IntoResponse {
    // 1. Validate title
    let raw_title = match payload.title {
        Some(ref t) => t.as_str(),
        None => {
            warn!("save_note called without a title field");
            return err(StatusCode::UNPROCESSABLE_ENTITY, "title is required").into_response();
        }
    };

    let title = sanitise(raw_title);
    if title.is_empty() {
        warn!("save_note called with a blank title");
        return err(StatusCode::UNPROCESSABLE_ENTITY, "title must not be blank").into_response();
    }

    // 2. Normalise content
    let content = payload
        .content
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_string();

    // 3. Resolve ID (client-supplied or fresh UUID)
    let id = payload
        .id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // 4. Write to store
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
    };

    info!(note_id = %note.id, title = %note.title, "note saved");

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

// ── GET /api/notes ───────────────────────────────────────────────────────────

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

// ── DELETE /api/notes/:id ────────────────────────────────────────────────────

pub async fn delete_note(
    State(store): State<NoteStore>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let id = id.trim().to_string();

    if id.is_empty() {
        warn!("delete_note called with empty id");
        return err(StatusCode::BAD_REQUEST, "note id must not be empty").into_response();
    }

    let mut map = match store.try_write() {
        Ok(guard) => guard,
        Err(_) => {
            error!("NoteStore write lock is poisoned / contended on delete");
            return err(
                StatusCode::SERVICE_UNAVAILABLE,
                "note store temporarily unavailable, please retry",
            )
            .into_response();
        }
    };

    if map.remove(&id).is_some() {
        info!(note_id = %id, "note deleted");
        (StatusCode::OK, Json(serde_json::json!({ "success": true, "id": id }))).into_response()
    } else {
        warn!(note_id = %id, "delete_note: id not found");
        err(StatusCode::NOT_FOUND, format!("no note with id \"{id}\"")).into_response()
    }
}
