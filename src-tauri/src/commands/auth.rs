use crate::backend::auth::{cadastrar_usuario, autenticar_usuario};
use crate::CommandResult;

#[tauri::command]
pub fn cadastrar_usuario_tauri(username: String, password: String) -> CommandResult<()> {
    match cadastrar_usuario(username, password) {
        Ok(_) => CommandResult { ok: true, data: Some(()), error: None },
        Err(e) => {
            eprintln!("Erro cadastrar_usuario: {}", e);
            CommandResult { ok: false, data: None, error: Some(e) }
        }
    }
}

#[tauri::command]
pub fn autenticar_usuario_tauri(username: String, password: String) -> CommandResult<()> {
    match autenticar_usuario(username, password) {
        Ok(_) => CommandResult { ok: true, data: Some(()), error: None },
        Err(e) => {
            eprintln!("Erro autenticar_usuario: {}", e);
            CommandResult { ok: false, data: None, error: Some(e) }
        }
    }
}
