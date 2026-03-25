#[cfg(test)]
mod tests {
    use crate::commands::auth::{register_user, authenticate_user};

    #[test]
    fn test_register_user_invalido() {
        let result = register_user("".to_string(), "".to_string());
        assert!(!result.ok);
    }

    #[test]
    fn test_authenticate_user_invalido() {
        let result = authenticate_user("".to_string(), "".to_string());
        assert!(!result.ok);
    }
}
use crate::backend::auth::{cadastrar_usuario, autenticar_usuario};
use crate::commands::download_manager::CommandResult;

#[tauri::command(rename = "register_user")]
pub fn register_user(username: String, password: String) -> CommandResult<()> {
    match cadastrar_usuario(username, password) {
        Ok(_) => CommandResult { ok: true, data: Some(()), error: None },
        Err(e) => {
            let msg = match e.as_str() {
                "Nome de usuário não pode ser vazio" => "O nome de usuário não pode ser vazio.",
                "Senha não pode ser vazia" => "A senha não pode ser vazia.",
                "Usuário já cadastrado" => "Usuário já cadastrado.",
                _ => "Erro ao registrar usuário. Tente novamente.",
            };
            eprintln!("Erro register_user: {}", msg);
            CommandResult { ok: false, data: None, error: Some(msg.to_string()) }
        }
    }
}

#[tauri::command(rename = "authenticate_user")]
pub fn authenticate_user(username: String, password: String) -> CommandResult<()> {
    match autenticar_usuario(username, password) {
        Ok(_) => CommandResult { ok: true, data: Some(()), error: None },
        Err(e) => {
            let msg = match e.as_str() {
                "Usuário não encontrado" => "Usuário não encontrado.",
                "Senha incorreta" => "Senha incorreta.",
                _ => "Erro ao autenticar usuário. Tente novamente.",
            };
            eprintln!("Erro authenticate_user: {}", msg);
            CommandResult { ok: false, data: None, error: Some(msg.to_string()) }
        }
    }
}
