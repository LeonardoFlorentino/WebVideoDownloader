use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MainUrl {
    pub url: String,
    pub filename: String,
    pub status: String, // "pendente" ou "concluído"
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
