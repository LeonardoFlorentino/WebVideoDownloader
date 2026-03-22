use crate::backend::auth::{cadastrar_usuario, autenticar_usuario};
use crate::CommandResult;

#[tauri::command(rename = "register_user")]
pub fn register_user(username: String, password: String) -> CommandResult<()> {
    match cadastrar_usuario(username, password) {
        Ok(_) => CommandResult { ok: true, data: Some(()), error: None },
        Err(e) => {
            eprintln!("Erro register_user: {}", e);
            CommandResult { ok: false, data: None, error: Some(e) }
        }
    }
}

#[tauri::command(rename = "authenticate_user")]
pub fn authenticate_user(username: String, password: String) -> CommandResult<()> {
    match autenticar_usuario(username, password) {
        Ok(_) => CommandResult { ok: true, data: Some(()), error: None },
        Err(e) => {
            eprintln!("Erro authenticate_user: {}", e);
            CommandResult { ok: false, data: None, error: Some(e) }
        }
    }
}
