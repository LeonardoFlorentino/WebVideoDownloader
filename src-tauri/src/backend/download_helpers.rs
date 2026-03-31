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
    should_stop: Option<&std::sync::Arc<std::sync::atomic::AtomicBool>>,
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
        let result = download_hls_file(client, url, dest_path, window, progress_event, id, username, should_stop);
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
                    if let Err(_e) = crate::backend::user_service::update_main_url_progress(
                        username.to_string(),
                        m3u8_url.to_string(),
                        1.0,
                    ) {
                        // log removido
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
    // Evento e log: preparando download (calculando tamanho total)
    println!("[DEBUG] Calculando tamanho total dos segmentos .ts (HEAD requests)...");
    if let Some(w) = window {
        let _ = w.emit("download-progress", serde_json::json!({
            "id": id_value,
            "progress": 0u64,
            "total": 0u64,
            "status": "preparando"
        }));
    }
    // Calcula o tamanho total real dos segmentos
    let mut total_size = 0u64;
    for seginfo in &segments {
        let head_resp = client.head(&seginfo.url).send();
        if let Ok(resp) = head_resp {
            if let Some(len) = resp.headers().get(reqwest::header::CONTENT_LENGTH) {
                if let Ok(size) = len.to_str().unwrap_or("0").parse::<u64>() {
                    total_size += size;
                }
            }
        }
    }
    println!("[DEBUG] Tamanho total calculado: {} bytes", total_size);
    for (i, seginfo) in segments.iter().enumerate() {
        if let Some(stop) = should_stop {
            if stop.load(std::sync::atomic::Ordering::SeqCst) {
                println!("[DEBUG] Download pausado pelo usuário no segmento {}", i);
                return Err("Download pausado pelo usuário".to_string());
            }
        }
        println!("[DEBUG] Baixando segmento {} de {}: {}", i + 1, segments.len(), seginfo.url);
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
        // Garante que bytes_downloaded nunca diminui
        bytes_downloaded = bytes_downloaded.saturating_add(seg_len);
        println!("[DEBUG] Progresso: {}/{} bytes ({}%)", bytes_downloaded, total_size, if total_size > 0 { bytes_downloaded * 100 / total_size  } else { 0 });
        if let Some(w) = window {
            let _ = w.emit("download-progress", serde_json::json!({
                "id": id_value,
                "progress": bytes_downloaded,
                "total": total_size,
                "status": "baixando"
            }));
        }
    }
    let meta = std::fs::metadata(&temp_dest).map_err(|e| format!("Erro ao finalizar arquivo temporário: {}", e))?;
    if meta.len() == 0 {
        return Err("Arquivo HLS criado mas está vazio".to_string());
    }

    println!("[DEBUG] Download dos segmentos concluído. Iniciando conversão FFmpeg...");
    // Sinaliza para o frontend que está convertendo
    if let Some(w) = window {
        let _ = w.emit("download-progress", serde_json::json!({
            "id": id_value,
            "progress": bytes_downloaded,
            "total": total_size,
            "status": "convertendo"
        }));
    }
    // Persiste o status 'convertendo' para o polling do frontend
    crate::backend::download_progress::update_progress(
        m3u8_url,
        crate::backend::download_progress::DownloadProgress {
            url: m3u8_url.to_string(),
            filename: dest_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string(),
            total_size: total_size,
            downloaded: bytes_downloaded,
            status: "convertendo".to_string(),
            id: Some(id_value),
        }
    );


    // Canal para comunicação entre thread e principal
    use std::sync::mpsc;
    let (tx, rx) = mpsc::channel();
    let temp_dest_clone = temp_dest.clone();
    let dest_path_clone = dest_path.to_path_buf();
    let dest_path_for_thread = dest_path_clone.clone();
    let m3u8_url_clone = m3u8_url.to_string();
    let id_value_clone = id_value;
    std::thread::spawn(move || {
        println!("[DEBUG] FFmpeg iniciado: convertendo {} para {}", temp_dest_clone.display(), dest_path_for_thread.display());
        let ffmpeg_status = Command::new("ffmpeg")
            .args(&["-y", "-i", temp_dest_clone.to_str().unwrap(), "-c", "copy", dest_path_for_thread.to_str().unwrap()])
            .status();
        let result = match ffmpeg_status {
            Ok(status) if status.success() => {
                let _ = std::fs::remove_file(&temp_dest_clone);
                println!("[DEBUG] FFmpeg finalizado com sucesso!");
                Ok(())
            }
            Ok(status) => {
                println!("[DEBUG] FFmpeg falhou com código {}", status);
                Err(format!("FFmpeg falhou com código {}", status))
            }
            Err(e) => {
                println!("[DEBUG] Erro ao executar FFmpeg: {}", e);
                Err(format!("Erro ao executar FFmpeg: {}", e))
            }
        };
        // Envia resultado para thread principal
        let _ = tx.send(result);
    });

    // Aguarda thread terminar (não trava GUI pois backend não é a thread do GUI)
    // Se quiser não bloquear, pode usar async ou spawn_blocking
    match rx.recv() {
        Ok(Ok(())) => {
            // Sucesso: atualiza progresso, user.json e emite eventos
            let progress = crate::backend::download_progress::DownloadProgress {
                url: m3u8_url_clone.clone(),
                filename: dest_path_clone.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string(),
                total_size: bytes_downloaded,
                downloaded: bytes_downloaded,
                status: "concluído".to_string(),
                id: Some(id_value_clone),
            };
            crate::backend::download_progress::update_progress(&m3u8_url_clone, progress.clone());
            if let Some(w) = window {
                // Atualiza user.json (main_urls) também
                if let Some(username) = username {
                    if let Err(_e) = crate::backend::user_service::update_main_url_progress(
                        username.to_string(),
                        m3u8_url_clone.clone(),
                        1.0,
                    ) {
                        // log removido
                    }
                }
                let _ = w.emit("download-progress", serde_json::json!({
                    "id": id_value_clone,
                    "progress": bytes_downloaded,
                    "total": bytes_downloaded,
                    "status": "concluído"
                }));
                let _ = w.emit("download_finished", serde_json::json!({
                    "url": m3u8_url_clone,
                    "filename": dest_path_clone.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                    "status": "concluído"
                }));
            }
        }
        Ok(Err(_e)) => {
            // log removido
            // Aqui pode emitir evento de erro se desejar
        }
        Err(_e) => {
            // log removido
        }
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

/// Calcula o tamanho total de um HLS (.m3u8) somando o tamanho de todos os segmentos .ts
pub fn calcular_tamanho_hls(client: &Client, m3u8_url: &str) -> Result<u64, String> {
    let resp = client.get(m3u8_url)
        .header("Referer", "https://player.jmvstream.com/")
        .send().map_err(|e| format!("Erro ao baixar playlist: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}: {}", resp.status().as_u16(), resp.status()));
    }
    let text = resp.text().map_err(|e| format!("Erro ao ler playlist: {}", e))?;
    let base_url = m3u8_url.rsplit_once('/').map(|(base, _)| base).unwrap_or("");
    let mut total_size = 0u64;
    let mut is_master_playlist = false;
    let mut variant_playlists: Vec<(u64, String)> = Vec::new();
    let mut last_bandwidth: Option<u64> = None;
    // Primeiro, detecta se é master playlist e coleta variantes
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
    }
    if is_master_playlist && !variant_playlists.is_empty() {
        // Seleciona a variante de maior banda e soma os segmentos dela (recursivo)
        let (_bw, url) = variant_playlists.iter().max_by_key(|(bw, _)| *bw).unwrap();
        return calcular_tamanho_hls(client, url);
    }
    // Não é master playlist, soma segmentos .ts normalmente
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        if !line.contains(".ts") { continue; }
        let ts_url = if line.starts_with("http") {
            line.to_string()
        } else {
            format!("{}/{}", base_url, line)
        };
        if let Ok(resp) = client.head(&ts_url).send() {
            if let Some(len) = resp.headers().get(reqwest::header::CONTENT_LENGTH) {
                if let Ok(size) = len.to_str().unwrap_or("0").parse::<u64>() {
                    total_size += size;
                }
            }
        }
    }
    Ok(total_size)
}

/// Baixa um vídeo JMVStream (.m3u8) com progresso e pausa dedicados
pub fn baixar_jmvstream(
    client: &Client,
    m3u8_url: &str,
    dest_path: &Path,
    window: Option<&Window>,
    id: Option<u64>,
    username: Option<&str>,
    should_stop: Option<&std::sync::Arc<std::sync::atomic::AtomicBool>>,
) -> Result<(), String> {
    // Depuração: início do download JMVStream
    println!("[DEBUG] Iniciando baixar_jmvstream para URL: {}", m3u8_url);
    // Emite e salva status 'preparando' antes de calcular o tamanho
    let filename = dest_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
    if let Some(w) = window {
        let _ = w.emit("download-progress", serde_json::json!({
            "id": id.unwrap_or(0),
            "progress": 0u64,
            "total": 0u64,
            "status": "preparando"
        }));
    }
    let progress_preparando = crate::backend::download_progress::DownloadProgress {
        url: m3u8_url.to_string(),
        filename: filename.clone(),
        total_size: 0,
        downloaded: 0,
        status: "preparando".to_string(),
        id,
    };
    crate::backend::download_progress::update_progress(m3u8_url, progress_preparando);

    // Calcula o tamanho total antes de iniciar
    let total_size = calcular_tamanho_hls(client, m3u8_url).unwrap_or(0);

    // Agora salva status 'baixando' com total_size correto
    let progress = crate::backend::download_progress::DownloadProgress {
        url: m3u8_url.to_string(),
        filename: filename.clone(),
        total_size,
        downloaded: 0,
        status: "baixando".to_string(),
        id,
    };
    crate::backend::download_progress::update_progress(m3u8_url, progress.clone());

    // Chama o fluxo padrão de download HLS, mas agora o progresso já tem o tamanho real
    download_hls_file(client, m3u8_url, dest_path, window, Some("download_progress"), id, username, should_stop)
}



