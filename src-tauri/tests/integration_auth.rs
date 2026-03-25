use web_video_downloader::commands::auth::{register_user, authenticate_user};

#[test]
fn test_register_and_authenticate_user() {
    let username = format!("integration_test_user_{}", rand::random::<u32>());
    let password = "integration_test_pass".to_string();
    let reg_result = register_user(username.clone(), password.clone());
    assert!(reg_result.ok, "Registro de usuário falhou: {:?}", reg_result.error);
    let auth_result = authenticate_user(username.clone(), password.clone());
    assert!(auth_result.ok, "Autenticação falhou: {:?}", auth_result.error);
}
