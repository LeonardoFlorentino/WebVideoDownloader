use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MainUrl {
    pub id: u64,
    pub url: String,
    pub filename: String,
    pub status: String, // "pendente" ou "concluído"
    pub progress: Option<f32>, // 0.0 a 1.0 (None = não iniciado)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub username: String,
    pub password: String,
    pub playlists: Vec<String>,
    pub main_urls: Vec<MainUrl>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct UserList {
    pub users: Vec<User>,
}
