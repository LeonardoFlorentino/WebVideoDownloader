use crate::backend::user_service::remove_main_url_by_id;

#[tauri::command]
pub fn remove_main_url_by_id_command(username: String, id: u64) -> CommandResult<()> {
    println!("[LOG] Chamada de remoção de main_url por id para usuário: '{}' | id: {}", username, id);
    match remove_main_url_by_id(username, id) {
        Ok(_) => CommandResult { ok: true, data: Some(()), error: None },
        Err(e) => {
            let msg = match e.as_str() {
                "Usuário não encontrado" => "Usuário não encontrado.",
                "ID não encontrado para remoção" => "ID não encontrado para remoção.",
                _ => "Erro ao remover URL. Tente novamente.",
            };
            eprintln!("Erro remove_main_url_by_id: {}", msg);
            return CommandResult { ok: false, data: None, error: Some(msg.to_string()) };
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::user::remove_main_url_command;

    #[test]
    fn test_remove_main_url_command_usuario_inexistente() {
        let username = "usuario_inexistente".to_string();
        let url = "url_inexistente".to_string();
        let result = remove_main_url_command(username, url);
        assert!(!result.ok);
        assert!(result.error.is_some());
    }
    // Adicione mais testes simulando chamadas das rotas
}

use crate::backend::user_service::remove_main_url;
#[tauri::command]
pub fn remove_main_url_command(username: String, url: String) -> CommandResult<()> {
    println!("[LOG] Chamada de remoção de main_url para usuário: '{}' | url: '{}'", username, url);
    match remove_main_url(username, url) {
        Ok(_) => CommandResult { ok: true, data: Some(()), error: None },
        Err(e) => {
            let msg = match e.as_str() {
                "Usuário não encontrado" => "Usuário não encontrado.",
                "URL não encontrada para remoção" => "URL não encontrada para remoção.",
                _ => "Erro ao remover URL. Tente novamente.",
            };
            eprintln!("Erro remove_main_url: {}", msg);
            CommandResult { ok: false, data: None, error: Some(msg.to_string()) }
        }
    }
}

use crate::backend::user_service::{add_main_url, get_main_urls, update_main_url_title};
use crate::commands::download_manager::CommandResult;

#[tauri::command]
pub fn add_main_url_command(username: String, url: String, filename: Option<String>) -> CommandResult<()> {
    match add_main_url(username, url, filename) {
        Ok(_) => CommandResult { ok: true, data: Some(()), error: None },
        Err(e) => {
            eprintln!("Erro add_main_url: {}", e);
            CommandResult { ok: false, data: None, error: Some(e) }
        }
    }
}

#[tauri::command]
pub fn get_main_urls_command(username: String) -> CommandResult<Vec<crate::backend::user::MainUrl>> {
    match get_main_urls(username) {
        Ok(urls) => CommandResult { ok: true, data: Some(urls), error: None },
        Err(e) => {
            eprintln!("Erro get_main_urls: {}", e);
            CommandResult { ok: false, data: None, error: Some(e) }
        }
    }
}

#[tauri::command(rename = "update_main_url_title")]
pub fn update_main_url_title_command(username: String, old_url: String, new_url: String, new_filename: String) -> CommandResult<()> {
    match update_main_url_title(username, old_url, new_url, new_filename) {
        Ok(_) => CommandResult { ok: true, data: Some(()), error: None },
        Err(e) => {
            eprintln!("Erro update_main_url_title: {}", e);
            CommandResult { ok: false, data: None, error: Some(e) }
        }
    }
}
