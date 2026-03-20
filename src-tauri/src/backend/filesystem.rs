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
    use std::fs;
    let mut path = get_project_root();
    path.push("Vídeos baixados");
    if !playlist.is_empty() {
        path.push(&playlist);
    }
    // Cria a pasta se não existir
    if let Err(e) = fs::create_dir_all(&path) {
        return Err(format!("Erro ao criar pasta: {}", e));
    }
    #[cfg(target_os = "windows")]
    {
        let path_str = path.to_str().ok_or_else(|| format!("Caminho inválido: {:?}", path))?;
        Command::new("explorer").arg(path_str).spawn().map_err(|e| {
            e.to_string()
        })?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(&path).spawn().map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(&path).spawn().map_err(|e| e.to_string())?;
    }
    Ok(())
}
