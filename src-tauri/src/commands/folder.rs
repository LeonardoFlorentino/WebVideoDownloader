use crate::backend::filesystem::open_download_folder;
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
