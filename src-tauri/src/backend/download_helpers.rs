use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom};
use tauri::Emitter;
use reqwest::blocking::Client;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tauri::Window;
use serde_json::json;

/// Baixa um arquivo HLS (.m3u8) e salva no destino, emitindo progresso via callback opcional
pub fn download_hls_file(
    client: &Client,
    m3u8_url: &str,
    dest_path: &Path,
    window: Option<&Window>,
    progress_event: Option<&str>,
) -> Result<(), String> {
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
        return download_hls_file(client, url, dest_path, window, progress_event);
    }
    if ts_urls.is_empty() {
        return Err("Nenhum segmento .ts encontrado na playlist".to_string());
    }
    let mut file = File::create(dest_path).map_err(|e| format!("Erro ao criar arquivo: {}", e))?;
    for (i, ts_url) in ts_urls.iter().enumerate() {
        let seg = client.get(ts_url)
            .send().map_err(|e| format!("Erro ao baixar segmento: {}", e))?
            .bytes().map_err(|e| format!("Erro ao ler segmento: {}", e))?;
        file.write_all(&seg).map_err(|e| format!("Erro ao salvar segmento: {}", e))?;
        if let (Some(w), Some(event)) = (window, progress_event) {
            let progress = ((i + 1) as f64 / ts_urls.len() as f64 * 100.0).round() as u8;
            let _ = w.emit(event, json!({ "url": m3u8_url, "progress": progress }));
        }
    }
    let meta = std::fs::metadata(dest_path).map_err(|e| format!("Erro ao finalizar arquivo: {}", e))?;
    if meta.len() == 0 {
        return Err("Arquivo HLS criado mas está vazio".to_string());
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


