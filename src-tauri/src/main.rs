mod backend;
mod commands;
use std::sync::Arc;
use crate::backend::download_manager::DownloadManager;
use commands::video::{
    download_video,
    download_cascade,
    list_downloaded_videos,
    get_title_from_url_command,
    start_download,
    integrated_pause_download,
    pausar_download,
};
use commands::auth::{register_user, authenticate_user};
use commands::user::{add_main_url_command, get_main_urls_command, update_main_url_title_command, remove_main_url_command};
use commands::remove_main_url_by_id_command;
use commands::playlist::{save_playlist, mark_playlist_downloaded};
use commands::folder::open_download_folder_tauri;




use commands::download_manager::{pause_download, resume_download};

fn main() {
    let manager = Arc::new(DownloadManager::default());

    tauri::Builder::default()
        .manage(manager)
        .invoke_handler(tauri::generate_handler![
            download_video,
            download_cascade,
            list_downloaded_videos,
            get_title_from_url_command,
            register_user,
            authenticate_user,
            add_main_url_command,
            get_main_urls_command,
            update_main_url_title_command,
            save_playlist,
            mark_playlist_downloaded,
            open_download_folder_tauri,
            remove_main_url_command,
            remove_main_url_by_id_command,
            pausar_download,
            start_download,
            pause_download,
            resume_download,
            integrated_pause_download,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
mod main_url_title_from_html;

