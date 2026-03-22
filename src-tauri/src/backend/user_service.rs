pub fn remove_main_url(username: String, url: String) -> Result<(), String> {
    let path = crate::backend::filesystem::get_project_root().join("user_data/user.json");
    let mut user_list = read_user_list(&path)?;
    let user = find_user_mut(&mut user_list, &username).ok_or("Usuário não encontrado")?;
    let original_len = user.main_urls.len();
    user.main_urls.retain(|mu| mu.url != url);
    if user.main_urls.len() == original_len {
        return Err("URL não encontrada para remoção".to_string());
    }
    write_user_list(&path, &user_list)?;
    Ok(())
}

pub fn get_main_urls(username: String) -> Result<Vec<MainUrl>, String> {
    let path = crate::backend::filesystem::get_project_root().join("user_data/user.json");
    let user_list = read_user_list(&path)?;
    let user = user_list.users.iter().find(|u| u.username == username).ok_or("Usuário não encontrado")?;
    Ok(user.main_urls.clone())
}

pub fn update_main_url_status(username: String, url: String, status: String) -> Result<(), String> {
    let path = crate::backend::filesystem::get_project_root().join("user_data/user.json");
    let mut user_list = read_user_list(&path)?;
    let user = find_user_mut(&mut user_list, &username).ok_or("Usuário não encontrado")?;
    let main_url = find_main_url_mut(user, &url).ok_or("URL não encontrada para o usuário")?;
    main_url.status = status;
    write_user_list(&path, &user_list)?;
    Ok(())
}
use crate::backend::user::{UserList, MainUrl};

use std::fs;
use serde_json;

fn read_user_list(path: &std::path::Path) -> Result<UserList, String> {
    if let Ok(data) = fs::read_to_string(path) {
        serde_json::from_str(&data).map_err(|e| e.to_string())
    } else {
        Ok(UserList::default())
    }
}

fn write_user_list(path: &std::path::Path, user_list: &UserList) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Erro ao criar diretório: {}", e))?;
    }
    let json = serde_json::to_string_pretty(user_list).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

fn find_user_mut<'a>(user_list: &'a mut UserList, username: &str) -> Option<&'a mut crate::backend::user::User> {
    user_list.users.iter_mut().find(|u| u.username == username)
}

fn find_main_url_mut<'a>(user: &'a mut crate::backend::user::User, url: &str) -> Option<&'a mut MainUrl> {
    user.main_urls.iter_mut().find(|mu| mu.url == url)
}

pub fn update_main_url_title(username: String, old_url: String, new_url: String, new_filename: String) -> Result<(), String> {
    let path = crate::backend::filesystem::get_project_root().join("user_data/user.json");
    let mut user_list = read_user_list(&path)?;
    let user = find_user_mut(&mut user_list, &username).ok_or("Usuário não encontrado")?;
    let main_url = find_main_url_mut(user, &old_url).ok_or("URL não encontrada para o usuário")?;
    main_url.url = new_url.clone();
    main_url.filename = new_filename.clone();
    write_user_list(&path, &user_list)?;
    Ok(())
}


pub fn add_main_url(username: String, url: String, filename: Option<String>) -> Result<(), String> {
    let path = crate::backend::filesystem::get_project_root().join("user_data/user.json");
    let mut user_list = if let Ok(list) = read_user_list(&path) { list } else { UserList::default() };
    let user = find_user_mut(&mut user_list, &username).ok_or("Usuário não encontrado")?;
    if !user.main_urls.iter().any(|mu| mu.url == url) {
        user.main_urls.push(MainUrl {
            url: url.clone(),
            filename: filename.clone().unwrap_or_else(|| "video".to_string()),
            status: "pendente".to_string(),
        });
        write_user_list(&path, &user_list)?;
        Ok(())
    } else {
        Err("URL já existe para o usuário".to_string())
    }
}


