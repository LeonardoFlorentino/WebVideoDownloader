use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PanelPlaylistLink {
    pub id: String,
    pub url: String,
    pub filename: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PanelPlaylist {
    pub id: String,
    pub name: String,
    pub links: Vec<PanelPlaylistLink>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PanelPlaylistUserEntry {
    pub username: String,
    pub playlists: Vec<PanelPlaylist>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PanelPlaylistStore {
    pub users: Vec<PanelPlaylistUserEntry>,
}