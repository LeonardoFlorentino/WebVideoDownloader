use crate::backend::user_service::remove_main_url;
#[allow(dead_code)]
#[tauri::command(rename = "remove_main_url")]
pub fn remove_main_url_command(username: String, url: String) -> CommandResult<()> {
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

#[tauri::command(rename = "add_main_url")]
pub fn add_main_url_command(username: String, url: String, filename: Option<String>) -> CommandResult<()> {
    match add_main_url(username, url, filename) {
        Ok(_) => CommandResult { ok: true, data: Some(()), error: None },
        Err(e) => {
            eprintln!("Erro add_main_url: {}", e);
            CommandResult { ok: false, data: None, error: Some(e) }
        }
    }
}

#[tauri::command(rename = "get_main_urls")]
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
