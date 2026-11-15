use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Note {
    pub id: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
}
