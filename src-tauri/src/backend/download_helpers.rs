use aes::Aes128;
use block_modes::{BlockMode, Cbc, block_padding::Pkcs7};
use hex::decode as hex_decode;
type Aes128Cbc = Cbc<Aes128, Pkcs7>;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom};
use tauri::Emitter;
use reqwest::blocking::Client;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tauri::Window;

/// Baixa um arquivo HLS (.m3u8) e salva no destino, emitindo progresso via callback opcional
pub fn download_hls_file(
    client: &Client,
    m3u8_url: &str,
    dest_path: &Path,
    window: Option<&Window>,
    progress_event: Option<&str>,
    id: Option<u64>,
    username: Option<&str>,
) -> Result<(), String> {
    let resp = client.get(m3u8_url)
        .header("Referer", "https://player.jmvstream.com/")
        .send().map_err(|e| format!("Erro ao baixar playlist: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}: {}", resp.status().as_u16(), resp.status()));
    }
    let text = resp.text().map_err(|e| format!("Erro ao ler playlist: {}", e))?;
    let base_url = m3u8_url.rsplit_once('/').map(|(base, _)| base).unwrap_or("");

    // Salva em arquivo temporário .ts
    let mut temp_dest = dest_path.to_path_buf();
    temp_dest.set_extension("ts");
    struct SegmentInfo {
        url: String,
        key_uri: Option<String>,
        iv: Option<Vec<u8>>,
    }
    let mut segments: Vec<SegmentInfo> = Vec::new();
    let mut is_master_playlist = false;
    let mut variant_playlists: Vec<(u64, String)> = Vec::new();
    let mut last_bandwidth: Option<u64> = None;
    let mut current_key_uri: Option<String> = None;
    let mut current_iv: Option<Vec<u8>> = None;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("#EXT-X-KEY:") {
            let method = line.split("METHOD=").nth(1).and_then(|s| s.split(',').next());
            match method {
                Some("AES-128") => {
                    if let Some(uri_part) = line.split("URI=").nth(1) {
                        let uri = uri_part.split(',').next().unwrap_or("").trim_matches('"');
                        current_key_uri = Some(if uri.starts_with("http") { uri.to_string() } else { format!("{}/{}", base_url, uri) });
                    }
                    if let Some(iv_part) = line.split("IV=").nth(1) {
                        let iv_str = iv_part.split(',').next().unwrap_or("").trim();
                        let iv = if iv_str.starts_with("0x") || iv_str.starts_with("0X") {
                            hex_decode(&iv_str[2..]).unwrap_or(vec![0; 16])
                        } else {
                            iv_str.as_bytes().to_vec()
                        };
                        current_iv = Some(iv);
                    } else {
                        current_iv = None;
                    }
                }
                Some("NONE") => {
                    current_key_uri = None;
                    current_iv = None;
                }
                _ => {}
            }
            continue;
        }
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
        segments.push(SegmentInfo {
            url: ts_url,
            key_uri: current_key_uri.clone(),
            iv: current_iv.clone(),
        });
    }
    use std::collections::HashMap;
    let mut key_cache: HashMap<String, Vec<u8>> = HashMap::new();
    if segments.is_empty() && is_master_playlist && !variant_playlists.is_empty() {
        let (_bw, url) = variant_playlists.iter().max_by_key(|(bw, _)| *bw).unwrap();
        let result = download_hls_file(client, url, dest_path, window, progress_event, id, username);
        if result.is_ok() {
            let meta = std::fs::metadata(dest_path).map_err(|e| format!("Erro ao finalizar arquivo: {}", e)).unwrap();
            let bytes_downloaded = meta.len();
            let id_value = id.or_else(|| {
                dest_path.file_stem().and_then(|s| s.to_str()).and_then(|stem| stem.split('_').last().and_then(|n| n.parse::<u64>().ok()))
            }).unwrap_or(0);
            let progress = crate::backend::download_progress::DownloadProgress {
                url: m3u8_url.to_string(),
                filename: dest_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string(),
                total_size: bytes_downloaded,
                downloaded: bytes_downloaded,
                status: "concluído".to_string(),
                id: Some(id_value),
            };
            crate::backend::download_progress::update_progress(m3u8_url, progress.clone());
            // Atualiza user.json (main_urls) para a URL principal
            if let Some(w) = window {
                // Usa username propagado corretamente
                if let Some(username) = username {
                    if let Err(e) = crate::backend::user_service::update_main_url_progress(
                        username.to_string(),
                        m3u8_url.to_string(),
                        1.0,
                    ) {
                        println!("[BACKEND][ERRO] Falha ao atualizar main_url_progress para usuário '{}': {}", username, e);
                    }
                }
                let _ = w.emit("download-progress", serde_json::json!({
                    "id": id_value,
                    "progress": bytes_downloaded,
                    "total": bytes_downloaded,
                    "status": "concluído"
                }));
                let _ = w.emit("download_finished", serde_json::json!({
                    "url": m3u8_url,
                    "filename": dest_path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                    "status": "concluído"
                }));
            }
        }
        return result;
    }
    if segments.is_empty() {
        return Err("Nenhum segmento .ts encontrado na playlist".to_string());
    }
    let mut file = File::create(&temp_dest).map_err(|e| format!("Erro ao criar arquivo temporário: {}", e))?;
    let mut bytes_downloaded = 0u64;
    let id_value = id.or_else(|| {
        dest_path.file_stem().and_then(|s| s.to_str()).and_then(|stem| stem.split('_').last().and_then(|n| n.parse::<u64>().ok()))
    }).unwrap_or(0);
    for (i, seginfo) in segments.iter().enumerate() {
        let seg = client.get(&seginfo.url)
            .send().map_err(|e| format!("Erro ao baixar segmento: {}", e))?
            .bytes().map_err(|e| format!("Erro ao ler segmento: {}", e))?;
        let mut seg_data = seg.to_vec();
        if let Some(ref key_url) = seginfo.key_uri {
            if !key_url.is_empty() {
                let key = if let Some(cached) = key_cache.get(key_url) {
                    cached.clone()
                } else {
                    let resp = client.get(key_url)
                        .send().map_err(|e| format!("Erro ao baixar chave AES: {}", e))?;
                    let key_bytes = resp.bytes().map_err(|e| format!("Erro ao ler chave AES: {}", e))?.to_vec();
                    key_cache.insert(key_url.clone(), key_bytes.clone());
                    key_bytes
                };
                let iv = if let Some(ref iv_bytes) = seginfo.iv {
                    iv_bytes.clone()
                } else {
                    let mut iv = vec![0u8; 16];
                    let idx_bytes = (i as u128).to_be_bytes();
                    iv[16 - idx_bytes.len()..].copy_from_slice(&idx_bytes);
                    iv
                };
                let cipher = Aes128Cbc::new_from_slices(&key, &iv).map_err(|e| format!("Erro ao criar cipher: {:?}", e))?;
                seg_data = cipher.decrypt_vec(&seg_data).map_err(|e| format!("Erro ao descriptografar segmento: {:?}", e))?;
            }
        }
        let seg_len = seg_data.len() as u64;
        file.write_all(&seg_data).map_err(|e| format!("Erro ao salvar segmento: {}", e))?;
        bytes_downloaded += seg_len;
        if let Some(w) = window {
            let _ = w.emit("download-progress", serde_json::json!({
                "id": id_value,
                "progress": bytes_downloaded,
                "total": segments.len() as u64 * 188000u64,
                "status": "baixando"
            }));
        }
    }
    let meta = std::fs::metadata(&temp_dest).map_err(|e| format!("Erro ao finalizar arquivo temporário: {}", e))?;
    if meta.len() == 0 {
        return Err("Arquivo HLS criado mas está vazio".to_string());
    }

    // Pós-processamento: remuxar para MP4 real usando FFmpeg
    let ffmpeg_status = Command::new("ffmpeg")
        .args(&["-y", "-i", temp_dest.to_str().unwrap(), "-c", "copy", dest_path.to_str().unwrap()])
        .status();
    match ffmpeg_status {
        Ok(status) if status.success() => {
            // Remove arquivo temporário .ts
            let _ = std::fs::remove_file(&temp_dest);
        }
        Ok(status) => {
            return Err(format!("FFmpeg falhou com código {} ao remuxar para MP4", status));
        }
        Err(e) => {
            return Err(format!("Erro ao executar FFmpeg: {}", e));
        }
    }
    // Atualiza status para concluido e emite evento de finalização
    // Atualiza progresso persistente
    let progress = crate::backend::download_progress::DownloadProgress {
        url: m3u8_url.to_string(),
        filename: dest_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string(),
        total_size: bytes_downloaded,
        downloaded: bytes_downloaded,
        status: "concluído".to_string(),
        id: Some(id_value),
    };
    crate::backend::download_progress::update_progress(m3u8_url, progress.clone());
    // Atualiza user.json (main_urls) também
    // username deve ser passado corretamente para esta função!
    if let Some(w) = window {
        // Usa username propagado corretamente
        if let Some(username) = username {
            if let Err(e) = crate::backend::user_service::update_main_url_progress(
                username.to_string(),
                m3u8_url.to_string(),
                1.0,
            ) {
                println!("[BACKEND][ERRO] Falha ao atualizar main_url_progress para usuário '{}': {}", username, e);
            }
        }
        let _ = w.emit("download-progress", serde_json::json!({
            "id": id_value,
            "progress": bytes_downloaded,
            "total": bytes_downloaded,
            "status": "concluído"
        }));
        let _ = w.emit("download_finished", serde_json::json!({
            "url": m3u8_url,
            "filename": dest_path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
            "status": "concluído"
        }));
    }
    Ok(())
}


/// Helper to open a file for appending, creating if needed
pub fn open_file_append(path: &Path) -> Result<File, String> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("Failed to open file: {}", e))
}

/// Helper to seek to the end of a file
pub fn seek_file_end(file: &mut File, offset: u64) -> Result<(), String> {
    file.seek(SeekFrom::Start(offset))
        .map(|_| ())
        .map_err(|e| format!("Failed to seek file: {}", e))
}


