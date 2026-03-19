use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Playlist {
    pub title: String,
    pub videos: Vec<String>,
}
