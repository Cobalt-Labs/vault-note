use serde::Deserialize;

#[derive(Deserialize)]
pub struct Note {
    pub id: Option<String>,
    pub title: Option<String>,
    pub _content: Option<String>,
}
