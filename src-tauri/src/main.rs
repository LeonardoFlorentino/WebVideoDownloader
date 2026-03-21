use tauri::Manager;
mod window_utils;
mod commands;
use commands::auth::{cadastrar_usuario_tauri, autenticar_usuario_tauri};
use commands::video::{baixar_video_tauri, baixar_em_cascata, listar_videos_baixados_tauri, get_title_from_url_tauri};
use commands::user::{add_main_url_tauri, get_main_urls_tauri, update_main_url_title_tauri, remove_main_url_tauri};
use commands::playlist::{salvar_playlist_tauri, marcar_playlist_baixada_tauri};
use commands::folder::open_download_folder_tauri;
use serde::Serialize;

#[derive(Serialize)]
pub struct CommandResult<T> {
    pub ok: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            tauri::async_runtime::spawn(window_utils::ajustar_janela_metade_tela(window, Some(52.0)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            baixar_video_tauri,
            baixar_em_cascata,
            listar_videos_baixados_tauri,
            get_title_from_url_tauri,
            cadastrar_usuario_tauri,
            autenticar_usuario_tauri,
            add_main_url_tauri,
            get_main_urls_tauri,
            update_main_url_title_tauri,
            salvar_playlist_tauri,
            marcar_playlist_baixada_tauri,
            open_download_folder_tauri,
            remove_main_url_tauri,
            commands::video::pausar_download,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
mod main_url_title_from_html;
mod backend {
    pub mod user;
    pub mod user_service;
    pub mod auth;
    pub mod download_progress;
    pub mod filesystem;
    pub mod downloads;
    pub mod playlist;
    pub mod playlist_service;
    pub mod listing;
}

