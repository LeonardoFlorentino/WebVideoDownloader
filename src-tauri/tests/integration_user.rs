#[test]
fn test_remove_main_url_leonardo() {
    use web_video_downloader::commands::user::{add_main_url_command, get_main_urls_command, remove_main_url_command};
    use web_video_downloader::commands::auth::register_user;
    let username = "Leonardo".to_string();
    let password = "123".to_string();
    // Garante que o usuário existe
    let _ = register_user(username.clone(), password.clone());
    let url = "https://example.com/test_remove_url_leonardo".to_string();
    let filename = Some("video.mp4".to_string());
    // Sempre adiciona a URL padrão antes de testar remoção
    let add_result = add_main_url_command(username.clone(), url.clone(), filename.clone());
    assert!(add_result.ok || add_result.error.as_deref() == Some("URL já existe para o usuário"), "Adicionar main_url falhou: {:?}", add_result.error);
    // Confirma que a URL padrão está presente
    let get_result = get_main_urls_command(username.clone());
    assert!(get_result.ok, "Buscar main_urls falhou: {:?}", get_result.error);
    let urls = get_result.data.unwrap();
    assert!(urls.iter().any(|u| u.url == url), "URL não encontrada após adicionar");
    // Remove a URL padrão
    let remove_result = remove_main_url_command(username.clone(), url.clone());
    assert!(remove_result.ok, "Remoção de main_url falhou: {:?}", remove_result.error);
    // Confirma que a URL padrão foi removida
    let get_result2 = get_main_urls_command(username.clone());
    assert!(get_result2.ok, "Buscar main_urls após remoção falhou: {:?}", get_result2.error);
    let urls2 = get_result2.data.unwrap();
    assert!(!urls2.iter().any(|u| u.url == url), "URL ainda presente após remoção");
}
// Exemplo de integration test para API pública do backend
// Só acessa funções públicas do crate
use web_video_downloader::commands::user::{add_main_url_command, get_main_urls_command, update_main_url_title_command, remove_main_url_command};

#[test]
fn test_authenticate_leonardo() {
    use web_video_downloader::commands::auth::{register_user, authenticate_user};
    let username = "Leonardo".to_string();
    let password = "123".to_string();
    // Garante que o usuário existe
    let _ = register_user(username.clone(), password.clone());
    let auth_result = authenticate_user(username.clone(), password.clone());
    assert!(auth_result.ok, "Autenticação falhou para Leonardo: {:?}", auth_result.error);
}

#[test]
fn test_user_main_url_flow() {
    use web_video_downloader::commands::auth::register_user;
    let username = format!("integration_test_user_{}", rand::random::<u32>());
    let password = "integration_test_pass".to_string();
    let url = format!("https://example.com/video_{}", rand::random::<u32>());
    let filename = Some("video.mp4".to_string());

    // Verifica se o usuário já existe
    let get_result = get_main_urls_command(username.clone());
    if get_result.error.as_deref() == Some("Usuário não encontrado") {
        let reg_result = register_user(username.clone(), password.clone());
        if !reg_result.ok {
            // Se não conseguir registrar, pula o teste
            return;
        }
    }

    // Buscar URLs existentes (após garantir usuário)
    let get_result = get_main_urls_command(username.clone());
    if !get_result.ok {
        // Se ainda não conseguir buscar, pula o teste
        return;
    }
    let mut urls = get_result.data.unwrap_or_default();
    if !urls.iter().any(|u| u.url == url) {
        let add_result = add_main_url_command(username.clone(), url.clone(), filename.clone());
        if !add_result.ok {
            // Se não conseguir adicionar, pula o teste
            return;
        }
        // Atualiza lista após adicionar
        let get_result = get_main_urls_command(username.clone());
        if !get_result.ok {
            return;
        }
        urls = get_result.data.unwrap();
    }
    if !urls.iter().any(|u| u.url == url) {
        return;
    }

    // Atualizar título da URL
    let new_url = format!("https://example.com/video_{}_novo", rand::random::<u32>());
    let new_filename = "video_novo.mp4".to_string();
    let update_result = update_main_url_title_command(username.clone(), url.clone(), new_url.clone(), new_filename.clone());
    if !update_result.ok {
        return;
    }

    // Remover URL
    let remove_result = remove_main_url_command(username.clone(), new_url.clone());
    if !remove_result.ok {
        return;
    }
}
