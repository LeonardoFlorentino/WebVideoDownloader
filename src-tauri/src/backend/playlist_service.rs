// ...existing code...

pub fn marcar_playlist_baixada(title: String) -> Result<(), String> {
    let path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")).join("src-tauri/src/playlist.json");
    let mut playlists: Vec<crate::backend::playlist::Playlist> = if let Ok(data) = std::fs::read_to_string(&path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    };
    for pl in playlists.iter_mut() {
        if pl.title == title {
            // Supondo que Playlist tenha um campo downloaded: bool
            // pl.downloaded = true;
        }
    }
    let json = serde_json::to_string_pretty(&playlists).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn salvar_playlist(title: String) -> Result<(), String> {
    let path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")).join("src-tauri/src/playlist.json");
    let mut playlists: Vec<crate::backend::playlist::Playlist> = if let Ok(data) = std::fs::read_to_string(&path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    };
    if !playlists.iter().any(|pl| pl.title == title) {
        playlists.push(crate::backend::playlist::Playlist { title, videos: Vec::new() });
    }
    let json = serde_json::to_string_pretty(&playlists).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}
