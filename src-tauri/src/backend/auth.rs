use crate::backend::user::{UserList, User};

pub fn cadastrar_usuario(username: String, password: String) -> Result<(), String> {
    if username.trim().is_empty() {
        return Err("Nome de usuário não pode ser vazio".to_string());
    }
    if password.trim().is_empty() {
        return Err("Senha não pode ser vazia".to_string());
    }
    let path = crate::backend::filesystem::get_project_root().join("user_data/user.json");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Erro ao criar diretório: {}", e))?;
    }
    let mut user_list: UserList = if let Ok(data) = std::fs::read_to_string(&path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        UserList::default()
    };
    if user_list.users.iter().any(|u| u.username == username) {
        return Err("Usuário já cadastrado".to_string());
    }
    user_list.users.push(User { username, password, playlists: Vec::new(), main_urls: Vec::new() });
    let json = serde_json::to_string_pretty(&user_list).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn autenticar_usuario(username: String, password: String) -> Result<(), String> {
    let path = crate::backend::filesystem::get_project_root().join("user_data/user.json");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Erro ao criar diretório: {}", e))?;
    }
    let user_list: UserList = if let Ok(data) = std::fs::read_to_string(&path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        UserList::default()
    };
    if let Some(user) = user_list.users.iter().find(|u| u.username == username) {
        if user.password == password {
            Ok(())
        } else {
            Err("Senha incorreta".to_string())
        }
    } else {
        Err("Usuário não encontrado".to_string())
    }
}
