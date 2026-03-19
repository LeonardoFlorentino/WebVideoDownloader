use std::path::PathBuf;

pub fn get_project_root() -> PathBuf {
    // Procura o package.json para identificar a raiz do projeto
    let mut path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if path.join("package.json").exists() {
            return path;
        }
        if !path.pop() {
            break;
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn open_download_folder(playlist: String) -> Result<(), String> {
    use std::process::Command;
    let mut path = get_project_root();
    path.push("Vídeos baixados");
    path.push(&playlist);
    if path.exists() {
        #[cfg(target_os = "windows")]
        {
            Command::new("explorer").arg(path).spawn().map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open").arg(path).spawn().map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "macos")]
        {
            Command::new("open").arg(path).spawn().map_err(|e| e.to_string())?;
        }
        Ok(())
    } else {
        Err("Pasta não encontrada".to_string())
    }
}
