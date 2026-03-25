use web_video_downloader::commands::playlist::{save_playlist, mark_playlist_downloaded};

#[test]
fn test_save_and_mark_playlist() {
    let title = format!("integration_test_playlist_{}", rand::random::<u32>());
    let save_result = save_playlist(title.clone());
    assert!(save_result.ok, "Salvar playlist falhou: {:?}", save_result.error);
    let mark_result = mark_playlist_downloaded(title.clone());
    assert!(mark_result.ok, "Marcar playlist como baixada falhou: {:?}", mark_result.error);
}
