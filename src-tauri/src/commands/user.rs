use crate::backend::user_service::remove_main_url;
#[allow(dead_code)]
#[tauri::command]
pub fn remove_main_url_tauri(username: String, url: String) -> CommandResult<()> {
    match remove_main_url(username, url) {
        Ok(_) => CommandResult { ok: true, data: Some(()), error: None },
        Err(e) => {
            eprintln!("Erro remove_main_url: {}", e);
            CommandResult { ok: false, data: None, error: Some(e) }
        }
    }
}
use crate::backend::user_service::{add_main_url, get_main_urls, update_main_url_title};
use crate::CommandResult;

#[tauri::command]
pub fn add_main_url_tauri(username: String, url: String, filename: Option<String>) -> CommandResult<()> {
    match add_main_url(username, url, filename) {
        Ok(_) => CommandResult { ok: true, data: Some(()), error: None },
        Err(e) => {
            eprintln!("Erro add_main_url: {}", e);
            CommandResult { ok: false, data: None, error: Some(e) }
        }
    }
}

#[tauri::command]
pub fn get_main_urls_tauri(username: String) -> CommandResult<Vec<crate::backend::user::MainUrl>> {
    match get_main_urls(username) {
        Ok(urls) => CommandResult { ok: true, data: Some(urls), error: None },
        Err(e) => {
            eprintln!("Erro get_main_urls: {}", e);
            CommandResult { ok: false, data: None, error: Some(e) }
        }
    }
}

#[tauri::command]
pub fn update_main_url_title_tauri(username: String, old_url: String, new_url: String, new_filename: String) -> CommandResult<()> {
    match update_main_url_title(username, old_url, new_url, new_filename) {
        Ok(_) => CommandResult { ok: true, data: Some(()), error: None },
        Err(e) => {
            eprintln!("Erro update_main_url_title: {}", e);
            CommandResult { ok: false, data: None, error: Some(e) }
        }
    }
}
