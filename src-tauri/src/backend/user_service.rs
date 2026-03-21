#[allow(dead_code)]
pub fn remove_main_url(username: String, url: String) -> Result<(), String> {
    let path = crate::backend::filesystem::get_project_root().join("user_data/user.json");
    let user_list_result = std::fs::read_to_string(&path);
    let mut user_list: UserList = match user_list_result {
        Ok(data) => match serde_json::from_str(&data) {
            Ok(list) => list,
            Err(e) => {
                return Err(format!("Erro ao desserializar user.json: {}", e));
            }
        },
        Err(e) => {
            return Err(format!("Erro ao ler user.json: {}", e));
        }
    };
    let user_opt = user_list.users.iter_mut().find(|u| u.username == username);
    if user_opt.is_none() {
        return Err("Usuário não encontrado".to_string());
    }
    let user = user_opt.unwrap();
    let original_len = user.main_urls.len();
    // Remove ignorando parâmetros de query, espaços e case
    fn normalize_url(u: &str) -> String {
        let base = match u.find('?') {
            Some(idx) => &u[..idx],
            None => u
        };
        base.trim().to_lowercase()
    }
    let url_norm = normalize_url(&url);
    user.main_urls.retain(|mu| normalize_url(&mu.url) != url_norm);
    if user.main_urls.len() == original_len {
        return Err("URL não encontrada para remoção".to_string());
    }
    let json_result = serde_json::to_string_pretty(&user_list);
    match &json_result {
        Ok(_) => (),
        Err(e) => {
            return Err(format!("Erro ao serializar user_list: {}", e));
        }
    }
    let json = json_result.unwrap();
    let write_result = std::fs::write(&path, &json);
    match &write_result {
        Ok(_) => (),
        Err(e) => {
            return Err(format!("Erro ao salvar user.json: {}", e));
        }
    }
    Ok(())
}
pub fn update_main_url_status(username: String, url: String, status: String) -> Result<(), String> {
    let path = crate::backend::filesystem::get_project_root().join("user_data/user.json");
    let user_list_result = std::fs::read_to_string(&path);
    let mut user_list: UserList = match user_list_result {
        Ok(data) => match serde_json::from_str(&data) {
            Ok(list) => list,
            Err(e) => {
                return Err(format!("Erro ao desserializar user.json: {}", e));
            }
        },
        Err(e) => {
            return Err(format!("Erro ao ler user.json: {}", e));
        }
    };
    let user_opt = user_list.users.iter_mut().find(|u| u.username == username);
    if user_opt.is_none() {
        return Err("Usuário não encontrado".to_string());
    }
    let user = user_opt.unwrap();
    let main_url_opt = user.main_urls.iter_mut().find(|mu| mu.url == url);
    if main_url_opt.is_none() {
        return Err("URL não encontrada para o usuário".to_string());
    }
    let main_url = main_url_opt.unwrap();
    main_url.status = status;
    let json_result = serde_json::to_string_pretty(&user_list);
    match &json_result {
        Ok(_) => (),
        Err(e) => {
            return Err(format!("Erro ao serializar user_list: {}", e));
        }
    }
    let json = json_result.unwrap();
    let write_result = std::fs::write(&path, &json);
    match &write_result {
        Ok(_) => (),
        Err(e) => {
            return Err(format!("Erro ao salvar user.json: {}", e));
        }
    }
    Ok(())
}
use crate::backend::user::{UserList, MainUrl};

pub fn update_main_url_title(username: String, old_url: String, new_url: String, new_filename: String) -> Result<(), String> {
    let path = crate::backend::filesystem::get_project_root().join("user_data/user.json");
    let user_list_result = std::fs::read_to_string(&path);
    let mut user_list: UserList = match user_list_result {
        Ok(data) => match serde_json::from_str(&data) {
            Ok(list) => list,
            Err(e) => {
                return Err(format!("Erro ao desserializar user.json: {}", e));
            }
        },
        Err(e) => {
            return Err(format!("Erro ao ler user.json: {}", e));
        }
    };
    let user_opt = user_list.users.iter_mut().find(|u| u.username == username);
    if user_opt.is_none() {
        return Err("Usuário não encontrado".to_string());
    }
    let user = user_opt.unwrap();
    let main_url_opt = user.main_urls.iter_mut().find(|mu| mu.url == old_url);
    if main_url_opt.is_none() {
        return Err("URL não encontrada para o usuário".to_string());
    }
    let main_url = main_url_opt.unwrap();
    main_url.url = new_url.clone();
    main_url.filename = new_filename.clone();
    let json_result = serde_json::to_string_pretty(&user_list);
    match &json_result {
        Ok(_) => (),
        Err(e) => {
            return Err(format!("Erro ao serializar user_list: {}", e));
        }
    }
    let json = json_result.unwrap();
    let write_result = std::fs::write(&path, &json);
    match &write_result {
        Ok(_) => (),
        Err(e) => {
            return Err(format!("Erro ao salvar user.json: {}", e));
        }
    }
    Ok(())
}

pub fn get_main_urls(username: String) -> Result<Vec<MainUrl>, String> {
    let path = crate::backend::filesystem::get_project_root().join("user_data/user.json");
    let user_list: UserList = if let Ok(data) = std::fs::read_to_string(&path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        UserList::default()
    };
    if let Some(user) = user_list.users.iter().find(|u| u.username == username) {
        Ok(user.main_urls.clone())
    } else {
        // ...existing code...
        Err("Usuário não encontrado".to_string())
    }
}

pub fn add_main_url(username: String, url: String, filename: Option<String>) -> Result<(), String> {
    let path = crate::backend::filesystem::get_project_root().join("user_data/user.json");
    // ...existing code...
    let mut user_list: UserList = if let Ok(data) = std::fs::read_to_string(&path) {
        // ...existing code...
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        // ...existing code...
        UserList::default()
    };
    if let Some(user) = user_list.users.iter_mut().find(|u| u.username == username) {
        // ...existing code...
        if !user.main_urls.iter().any(|mu| mu.url == url) {
            // ...existing code...
            user.main_urls.push(MainUrl {
                url: url.clone(),
                filename: filename.clone().unwrap_or_else(|| "video".to_string()),
                status: "pendente".to_string(),
            });
        } else {
            // ...existing code...
        }
        let json = serde_json::to_string_pretty(&user_list).map_err(|e| e.to_string())?;
        std::fs::write(&path, json).map_err(|e| e.to_string())?;
        // ...existing code...
        Ok(())
    } else {
        // ...existing code...
        Err("Usuário não encontrado".to_string())
    }
}
