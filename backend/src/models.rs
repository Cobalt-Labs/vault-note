use serde::Deserialize;

#[derive(Deserialize)]
pub struct Note {
    pub title: String,
    pub _content: String,
}
