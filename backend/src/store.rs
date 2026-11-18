use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::Note;

pub type NoteStore = Arc<RwLock<HashMap<String, Note>>>;

pub fn new_store() -> NoteStore {
    Arc::new(RwLock::new(HashMap::new()))
}
