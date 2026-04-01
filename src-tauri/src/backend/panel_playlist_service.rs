use crate::backend::panel_playlist::{
    PanelPlaylist,
    PanelPlaylistStore,
    PanelPlaylistUserEntry,
};
use std::fs;
use std::path::PathBuf;

fn get_panel_playlist_path() -> PathBuf {
    crate::backend::filesystem::get_project_root().join("user_data/panel_playlists.json")
}

fn read_store(path: &std::path::Path) -> Result<PanelPlaylistStore, String> {
    if let Ok(data) = fs::read_to_string(path) {
        serde_json::from_str(&data).map_err(|e| e.to_string())
    } else {
        Ok(PanelPlaylistStore::default())
    }
}

fn write_store(path: &std::path::Path, store: &PanelPlaylistStore) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Erro ao criar diretório: {}", e))?;
    }

    let json = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

pub fn get_panel_playlists(username: String) -> Result<Vec<PanelPlaylist>, String> {
    let path = get_panel_playlist_path();
    let store = read_store(&path)?;

    Ok(store
        .users
        .into_iter()
        .find(|entry| entry.username == username)
        .map(|entry| entry.playlists)
        .unwrap_or_default())
}

pub fn replace_panel_playlists(username: String, playlists: Vec<PanelPlaylist>) -> Result<(), String> {
    let path = get_panel_playlist_path();
    let mut store = read_store(&path)?;

    if let Some(entry) = store.users.iter_mut().find(|entry| entry.username == username) {
        entry.playlists = playlists;
    } else {
        store.users.push(PanelPlaylistUserEntry { username, playlists });
    }

    write_store(&path, &store)
}