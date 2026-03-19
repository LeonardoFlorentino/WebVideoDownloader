use tauri::Emitter;
use tauri::Window;
// ...existing code...
use regex::Regex;
// ...existing code...

pub fn baixar_video(url: &str, filename: &str) -> Result<(), String> {
    let response = reqwest::blocking::get(url).map_err(|e| format!("Erro ao baixar: {}", e))?;
    if !response.status().is_success() {
        return Err(format!("HTTP {}: {}", response.status().as_u16(), response.status()));
    }
    let _total_size = response.content_length().unwrap_or(0);
    let ext = filename;
    let mut path = crate::backend::filesystem::get_project_root();
    path.push("Vídeos baixados");
    if let Some(content_type) = response.headers().get("content-type") {
        let ct = content_type.to_str().unwrap_or("");
        if !ct.starts_with("video/") && !ct.contains("octet-stream") {
            return Err(format!("O link não retorna um vídeo válido. Content-Type: {}", ct));
        }
    }
    if let Err(e) = std::fs::create_dir_all(&path) {
        return Err(format!("Erro ao criar pasta: {}", e));
    }
    path.push(ext);
    let mut file = match std::fs::File::create(&path) {
        Ok(f) => f,
        Err(e) => {
            return Err(format!("Erro ao criar arquivo: {}", e));
        }
    };
    let bytes = match response.bytes() {
        Ok(b) => b,
        Err(e) => {
            return Err(format!("Erro ao ler bytes: {}", e));
        }
    };
    if let Err(e) = std::io::Write::write_all(&mut file, &bytes) {
        return Err(format!("Erro ao salvar: {}", e));
    }
    match std::fs::metadata(&path) {
        Ok(meta) => {
            if meta.len() == 0 {
                return Err("Arquivo criado mas está vazio".to_string());
            }
        }
        Err(e) => {
            return Err(format!("Arquivo não criado corretamente: {}", e));
        }
    }
    Ok(())
}

pub fn baixar_hls_emit(window: &Window, m3u8_url: &str, filename: &str, id: Option<u64>) -> Result<(), String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| format!("Erro ao criar client: {}", e))?;
    let resp = client.get(m3u8_url)
        .header("Referer", "https://player.jmvstream.com/")
        .send().map_err(|e| format!("Erro ao baixar playlist: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}: {}", resp.status().as_u16(), resp.status()));
    }
    let text = resp.text().map_err(|e| format!("Erro ao ler playlist: {}", e))?;
    let base_url = m3u8_url.rsplit_once('/').map(|(base, _)| base).unwrap_or("");
    let mut ts_urls = Vec::new();
    let mut is_master_playlist = false;
    let mut variant_playlists: Vec<(u64, String)> = Vec::new();
    let mut last_bandwidth: Option<u64> = None;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("#EXT-X-STREAM-INF:") {
            is_master_playlist = true;
            if let Some(bw_str) = line.split("BANDWIDTH=").nth(1) {
                let bw = bw_str.split(',').next().unwrap_or("").parse::<u64>().unwrap_or(0);
                last_bandwidth = Some(bw);
            }
            continue;
        }
        if is_master_playlist && !line.is_empty() && !line.starts_with('#') {
            let url = if line.starts_with("http") {
                line.to_string()
            } else {
                format!("{}/{}", base_url, line)
            };
            variant_playlists.push((last_bandwidth.unwrap_or(0), url));
            last_bandwidth = None;
            continue;
        }
        if line.is_empty() || line.starts_with('#') { continue; }
        if !line.contains(".ts") { continue; }
        let ts_url = if line.starts_with("http") {
            line.to_string()
        } else {
            format!("{}/{}", base_url, line)
        };
        ts_urls.push(ts_url);
    }
    if ts_urls.is_empty() && is_master_playlist && !variant_playlists.is_empty() {
        let (_bw, url) = variant_playlists.iter().max_by_key(|(bw, _)| *bw).unwrap();
        return baixar_hls_emit(window, url, filename, id);
    }
    if ts_urls.is_empty() {
        return Err("Nenhum segmento .ts encontrado na playlist".to_string());
    }
    let mut path = crate::backend::filesystem::get_project_root();
    path.push("Vídeos baixados");
    if let Err(e) = std::fs::create_dir_all(&path) {
        return Err(format!("Erro ao criar pasta: {}", e));
    }
    path.push(filename);
    let mut file = match std::fs::File::create(&path) {
        Ok(f) => f,
        Err(e) => {
            return Err(format!("Erro ao criar arquivo: {}", e));
        }
    };
    for (i, ts_url) in ts_urls.iter().enumerate() {
        let seg = client.get(ts_url)
            .send().map_err(|e| format!("Erro ao baixar segmento: {}", e))?
            .bytes().map_err(|e| format!("Erro ao ler segmento: {}", e))?;
        std::io::Write::write_all(&mut file, &seg).map_err(|e| format!("Erro ao salvar segmento: {}", e))?;
        let progress = ((i + 1) as f64 / ts_urls.len() as f64 * 100.0).round() as u8;
        let _ = window.emit("download_progress", serde_json::json!({ "url": m3u8_url, "progress": progress }));
    }
    match std::fs::metadata(&path) {
        Ok(meta) => {
            if meta.len() == 0 {
                return Err("Arquivo HLS criado mas está vazio".to_string());
            }
        }
        Err(e) => {
            return Err(format!("Arquivo HLS não criado corretamente: {}", e));
        }
    }
    Ok(())
}

pub fn baixar_player_jmvstream(window: &Window, username: &str, player_url: &str, output: &str, id: Option<u64>) -> Result<(), String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| format!("Erro ao criar client: {}", e))?;
    let html = client.get(player_url)
        .header("Referer", player_url)
        .send().map_err(|e| format!("Erro ao baixar HTML do player: {}", e))?
        .text().map_err(|e| format!("Erro ao ler HTML do player: {}", e))?;
    let mut path = crate::backend::filesystem::get_project_root();
    path.push("user_data");
    std::fs::create_dir_all(&path).ok();
    path.push("last_player_html.html");
    let _ = std::fs::write(&path, &html);
    let re = Regex::new(r#"(src|file|source)"\s*:\s*"([^"]+\.m3u8[^"]*)"|https?://[^"]+\.m3u8[^"]*"#).unwrap();
    let m3u8_url = if let Some(cap) = re.captures(&html) {
        if let Some(url) = cap.get(2) {
            Some(url.as_str())
        } else {
            let full = cap.get(0).map(|m| m.as_str());
            full.and_then(|s| s.split('"').find(|x| x.contains(".m3u8")))
        }
    } else {
        None
    };
    let m3u8_url = match m3u8_url {
        Some(url) => url,
        None => return Err("Não foi possível extrair o link .m3u8 do player. O HTML foi salvo em user_data/last_player_html.html para análise.".to_string()),
    };
    let _ = crate::backend::user_service::update_main_url_title(
        username.to_string(),
        player_url.to_string(),
        m3u8_url.to_string(),
        output.to_string(),
    );
    baixar_hls_emit(window, m3u8_url, output, id)
}
