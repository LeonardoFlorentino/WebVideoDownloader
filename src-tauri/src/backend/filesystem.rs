use std::path::PathBuf;

pub fn get_project_root() -> PathBuf {
    // Procura o package.json a partir do diretório atual (onde o comando foi executado)
    let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if path.join("package.json").exists() {
            return path;
        }
        if !path.pop() {
            break;
        }
    }
    PathBuf::from(".")
}

pub fn open_download_folder(playlist: String) -> Result<(), String> {
    use std::process::Command;
    let mut path = get_project_root();
    path.push("Vídeos baixados");
    if !playlist.is_empty() {
        path.push(&playlist);
    }
    // Só abre se a pasta existir
    if !path.exists() {
        return Err(format!("A pasta não existe: {}", path.display()));
    }
    #[cfg(target_os = "windows")]
    {
        let dir = if path.is_file() {
            path.parent().map(|p| p.to_path_buf()).unwrap_or(path.clone())
        } else {
            path.clone()
        };
        let path_str = dir.to_str().ok_or_else(|| format!("Caminho inválido: {:?}", dir))?;
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
