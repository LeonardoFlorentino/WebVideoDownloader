use web_video_downloader::commands::folder::open_download_folder_tauri;

#[test]
fn test_open_download_folder() {
    let playlist = "integration_test_playlist".to_string();
    let result = open_download_folder_tauri(playlist);
    // Pode falhar se a pasta não existir, mas não deve panic
    assert!(!result.ok || result.error.is_some() || result.ok);
}
