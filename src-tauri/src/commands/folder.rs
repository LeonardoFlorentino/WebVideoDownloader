use crate::backend::filesystem::{ensure_download_folder, open_download_folder};
use crate::commands::download_manager::CommandResult;

#[tauri::command(rename = "open_download_folder")]
pub fn open_download_folder_tauri(playlist: String) -> CommandResult<()> {
    match open_download_folder(playlist.clone()) {
        Ok(_) => CommandResult { ok: true, data: Some(()), error: None },
        Err(e) => {
            eprintln!("Erro open_download_folder: {}", e);
            CommandResult {
                ok: false,
                data: None,
                error: Some(format!("Não foi possível abrir a pasta de downloads: {}", e)),
            }
        }
    }
}

#[tauri::command]
pub fn create_download_folder_tauri(playlist: String) -> CommandResult<String> {
    match ensure_download_folder(playlist) {
        Ok(path) => CommandResult {
            ok: true,
            data: Some(path.to_string_lossy().into_owned()),
            error: None,
        },
        Err(e) => {
            eprintln!("Erro create_download_folder: {}", e);
            CommandResult {
                ok: false,
                data: None,
                error: Some(format!("Não foi possível criar a pasta de downloads: {}", e)),
            }
        }
    }
}
