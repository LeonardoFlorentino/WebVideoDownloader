/// Busca o username associado a uma URL (primeiro usuário que possuir a url em main_urls)
pub fn get_username_for_url(url: &str) -> Option<String> {
    let path = crate::backend::filesystem::get_project_root().join("user_data/user.json");
    if let Ok(user_list) = read_user_list(&path) {
        for user in user_list.users {
            if user.main_urls.iter().any(|mu| mu.url == url) {
                return Some(user.username.clone());
            }
        }
    }
    None
}
/// Update the progress field for a MainUrl in user.json
pub fn update_main_url_progress(username: String, url: String, progress: f32) -> Result<(), String> {
    let path = crate::backend::filesystem::get_project_root().join("user_data/user.json");
    let mut user_list = read_user_list(&path)?;
    let user = find_user_mut(&mut user_list, &username).ok_or("Usuário não encontrado")?;
    let main_url = find_main_url_mut(user, &url).ok_or("URL não encontrada para o usuário")?;
    main_url.progress = Some(progress);
    // Atualiza status automaticamente
    if main_url.status != "concluído" {
        if progress >= 1.0 {
            main_url.status = "concluído".to_string();
        } else if progress > 0.0 {
            main_url.status = "pausado".to_string();
        } else {
            main_url.status = "pendente".to_string();
        }
    }
    write_user_list(&path, &user_list)?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_remove_main_url_nao_encontrada() {
        let username = "usuario_teste".to_string();
        let url = "url_inexistente".to_string();
        let result = remove_main_url(username, url);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_main_urls_usuario_nao_encontrado() {
        let username = "usuario_inexistente".to_string();
        let result = get_main_urls(username);
        assert!(result.is_err());
    }
    // Adicione mais testes para add_main_url, update_main_url_title, etc.
}

pub fn remove_main_url(username: String, url: String) -> Result<(), String> {
    let path = crate::backend::filesystem::get_project_root().join("user_data/user.json");
    let mut user_list = read_user_list(&path)?;
    let user = find_user_mut(&mut user_list, &username).ok_or("Usuário não encontrado")?;
    let url_trimmed = url.trim();
    let original_len = user.main_urls.len();
    user.main_urls.retain(|mu| mu.url.trim() != url_trimmed);
    if user.main_urls.len() == original_len {
        return Err("URL não encontrada para remoção".to_string());
    }
    // Remove progresso persistido também
    crate::backend::download_progress::remove_progress(url_trimmed);
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
    // Não sobrescrever se já está concluído
    if main_url.status != "concluído" {
        main_url.status = status;
    }
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
    let base_url = url.split('?').next().unwrap_or(url);
    user.main_urls.iter_mut().find(|mu| mu.url.split('?').next().unwrap_or(&mu.url) == base_url)
}

pub fn update_main_url_title(username: String, old_url: String, new_url: String, new_filename: String) -> Result<(), String> {
    let path = crate::backend::filesystem::get_project_root().join("user_data/user.json");
    let mut user_list = read_user_list(&path)?;
    let user = find_user_mut(&mut user_list, &username).ok_or("Usuário não encontrado")?;
    let main_url = find_main_url_mut(user, &old_url).ok_or("URL não encontrada para o usuário")?;
    let url_changed = main_url.url != new_url;
    main_url.url = new_url.clone();
    main_url.filename = new_filename.clone();
    // Se a URL mudou, o progresso anterior não tem mais relação com o novo conteúdo
    if url_changed {
        main_url.progress = Some(0.0);
        main_url.status = "pendente".to_string();
    }
    write_user_list(&path, &user_list)?;
    Ok(())
}


pub fn add_main_url(username: String, url: String, filename: Option<String>) -> Result<(), String> {
    let path = crate::backend::filesystem::get_project_root().join("user_data/user.json");
    // Try to read user_list, if corrupted, reset to default
    let mut user_list = match read_user_list(&path) {
        Ok(list) => list,
        Err(_) => UserList::default(),
    };
    // Auto-create user if not found
    let user = if let Some(u) = find_user_mut(&mut user_list, &username) {
        u
    } else {
        user_list.users.push(crate::backend::user::User {
            username: username.clone(),
            password: String::new(),
            playlists: Vec::new(),
            main_urls: Vec::new(),
        });
        user_list.users.last_mut().unwrap()
    };
    if !user.main_urls.iter().any(|mu| mu.url == url) {
        // Gera id autoincrementado único
        let next_id = user.main_urls.iter().map(|mu| mu.id).max().unwrap_or(0) + 1;
        user.main_urls.push(MainUrl {
            id: next_id,
            url: url.clone(),
            filename: filename.clone().unwrap_or_else(|| "video".to_string()),
            status: "pendente".to_string(),
            progress: None,
        });
        write_user_list(&path, &user_list)?;
        Ok(())
    } else {
        Err("URL já existe para o usuário".to_string())
    }
}

pub fn remove_main_url_by_id(username: String, id: u64) -> Result<(), String> {
    let path = crate::backend::filesystem::get_project_root().join("user_data/user.json");
    let mut user_list = read_user_list(&path)?;
    let user = find_user_mut(&mut user_list, &username).ok_or("Usuário não encontrado")?;
    let original_len = user.main_urls.len();
    // Captura a URL antes de remover
    let url_to_remove = user.main_urls.iter().find(|mu| mu.id == id).map(|mu| mu.url.clone());
    user.main_urls.retain(|mu| mu.id != id);
    if user.main_urls.len() == original_len {
        return Err("ID não encontrado para remoção".to_string());
    }
    // Remove progresso persistido também
    if let Some(url) = url_to_remove {
        crate::backend::download_progress::remove_progress(&url);
    }
    write_user_list(&path, &user_list)?;
    Ok(())
}


