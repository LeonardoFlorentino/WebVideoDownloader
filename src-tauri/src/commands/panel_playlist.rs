use crate::backend::panel_playlist::PanelPlaylist;
use crate::backend::panel_playlist_service::{
    get_panel_playlists,
    replace_panel_playlists,
};
use crate::commands::download_manager::CommandResult;

#[tauri::command]
pub fn get_panel_playlists_command(username: String) -> CommandResult<Vec<PanelPlaylist>> {
    match get_panel_playlists(username) {
        Ok(playlists) => CommandResult {
            ok: true,
            data: Some(playlists),
            error: None,
        },
        Err(error) => CommandResult {
            ok: false,
            data: None,
            error: Some(error),
        },
    }
}

#[tauri::command]
pub fn replace_panel_playlists_command(
    username: String,
    playlists: Vec<PanelPlaylist>,
) -> CommandResult<()> {
    match replace_panel_playlists(username, playlists) {
        Ok(_) => CommandResult {
            ok: true,
            data: Some(()),
            error: None,
        },
        Err(error) => CommandResult {
            ok: false,
            data: None,
            error: Some(error),
        },
    }
}