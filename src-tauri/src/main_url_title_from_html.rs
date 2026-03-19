use tauri::command;

#[command]
pub fn get_title_from_url(url: String) -> Result<String, String> {
    let html = match reqwest::blocking::get(&url).and_then(|r| r.text()) {
        Ok(h) => h,
        Err(e) => return Err(format!("Erro ao baixar HTML: {}", e)),
    };
    let title_re = regex::Regex::new(r#"<title>(.*?)</title>"#).unwrap();
    if let Some(cap) = title_re.captures(&html) {
        let title_raw = cap.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
        // Limpa caracteres inválidos para nome de arquivo
        let mut title = title_raw.replace(|c: char| r#"<>:"/|?*"#.contains(c), "_");
        // Se houver .mp4 no título, corta no primeiro .mp4
        if let Some(idx) = title.to_lowercase().find(".mp4") {
            title = title[..idx].to_string();
        }
        // Remove espaços finais
        let filename = title.trim_end().to_string();
        Ok(filename)
    } else {
        Err("Título não encontrado no HTML".to_string())
    }
}
