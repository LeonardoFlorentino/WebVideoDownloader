use reqwest::{blocking, Client};

#[allow(dead_code)]
pub fn create_async_client() -> Result<Client, String> {
    Client::builder()
        .user_agent("Mozilla/5.0")
        .build()
        .map_err(|e| format!("Erro ao criar client: {}", e))
}

#[allow(dead_code)]
pub fn create_blocking_client() -> Result<blocking::Client, String> {
    blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| format!("Erro ao criar client: {}", e))
}