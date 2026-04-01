use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};
use once_cell::sync::Lazy;
// Controle global de flags de pausa por URL
pub static PAUSE_FLAGS: Lazy<Mutex<HashMap<String, Arc<std::sync::atomic::AtomicBool>>>> = Lazy::new(|| Mutex::new(HashMap::new()));
// Controle global de downloads ativos por URL
static ACTIVE_HLS_DOWNLOADS: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));
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
    should_stop: Option<&Arc<std::sync::atomic::AtomicBool>>,
    progress_key: Option<&str>,
    source: Option<&str>,
) -> Result<(), String> {
    let progress_key = progress_key.unwrap_or(m3u8_url);
    let scope = source.unwrap_or("home");
    // Salva em arquivo temporário .ts
    let mut temp_dest = dest_path.to_path_buf();
    temp_dest.set_extension("ts");

    // Resume real e baseado no arquivo parcial em disco.
    let existing_temp_size = std::fs::metadata(&temp_dest).map(|m| m.len()).unwrap_or(0);
    let existing_progress = crate::backend::download_progress::get_progress(progress_key);
    let is_resuming = existing_temp_size > 0;
    
    // Só limpa progresso se for novo download, não resume
    if !is_resuming {
        crate::backend::download_progress::remove_progress(progress_key);
    } 
    
    // Impede múltiplas execuções concorrentes para a mesma URL
    {
        let mut active = ACTIVE_HLS_DOWNLOADS.lock().unwrap();
        if active.contains(progress_key) {
            return Err("Download já em andamento para esta URL".to_string());
        }
        active.insert(progress_key.to_string());
    }
    // Emite status 'preparando' imediatamente ao iniciar
    let id_value = id.unwrap_or(0);
    let preparing_json = serde_json::json!({
        "id": id_value,
        "progress": 0u64,
        "total": 0u64,
        "status": "preparando"
    });
    if let Some(w) = window {
        let _ = w.emit("download-progress", preparing_json.clone());
    }

    // Recupera ou cria flag de pausa global para esta URL
    let pause_flag = {
        let mut map = PAUSE_FLAGS.lock().unwrap();
        let flag = map
            .entry(progress_key.to_string())
            .or_insert_with(|| Arc::new(std::sync::atomic::AtomicBool::new(false)))
            .clone();
        flag
    };

    // Se a flag já estava em true quando chegamos aqui (pausa pedida durante "preparando"),
    // salva status pausado e retorna sem iniciar o download.
    if pause_flag.load(std::sync::atomic::Ordering::SeqCst) {
        let id_value_early = id.unwrap_or(0);
        let prog_prev = existing_progress.as_ref();
        let total_prev = prog_prev.map(|p| p.total_size).unwrap_or(0);
        let dl_prev = prog_prev.map(|p| p.downloaded).unwrap_or(existing_temp_size);
        crate::backend::download_progress::update_progress(
            progress_key,
            crate::backend::download_progress::DownloadProgress {
                url: m3u8_url.to_string(),
                filename: dest_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string(),
                total_size: total_prev,
                downloaded: dl_prev,
                status: "pausado".to_string(),
                id: Some(id_value_early),
                scope: Some(scope.to_string()),
            },
        );
        if let Some(w) = window {
            let _ = w.emit("download-progress", serde_json::json!({
                "id": id_value_early,
                "progress": dl_prev,
                "total": total_prev,
                "status": "pausado"
            }));
        }
        return Err("Download pausado pelo usuário".to_string());
    }

    // Só reseta a flag para false aqui porque já confirmamos que não há pausa pendente.
    pause_flag.store(false, std::sync::atomic::Ordering::SeqCst);

    // Ao final, sempre remove do set global
    struct ActiveGuard<'a> {
        url: &'a str,
    }
    impl<'a> Drop for ActiveGuard<'a> {
        fn drop(&mut self) {
            let mut active = ACTIVE_HLS_DOWNLOADS.lock().unwrap();
            active.remove(self.url);
        }
    }
    let _guard = ActiveGuard { url: progress_key };
    let resp = client.get(m3u8_url)
        .header("Referer", "https://player.jmvstream.com/")
        .send().map_err(|e| format!("Erro ao baixar playlist: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}: {}", resp.status().as_u16(), resp.status()));
    }
    let text = resp.text().map_err(|e| format!("Erro ao ler playlist: {}", e))?;
    let base_url = m3u8_url.rsplit_once('/').map(|(base, _)| base).unwrap_or("");

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
        let result = download_hls_file(client, url, dest_path, window, progress_event, id, username, should_stop, Some(progress_key), Some(scope));
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
                scope: Some(scope.to_string()),
            };
            crate::backend::download_progress::update_progress(progress_key, progress.clone());
            if let Some(w) = window {
                if scope == "home" {
                    if let Some(username) = username {
                        if let Err(_e) = crate::backend::user_service::update_main_url_progress(
                            username.to_string(),
                            m3u8_url.to_string(),
                            1.0,
                        ) {
                            // log removido
                        }
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
        // Emite status erro para o frontend
        if let Some(w) = window {
            let _ = w.emit("download-progress", serde_json::json!({
                "id": id.unwrap_or(0),
                "progress": 0u64,
                "total": 0u64,
                "status": "erro",
                "mensagem": "Nenhum segmento .ts encontrado na playlist"
            }));
        }
        crate::backend::download_progress::update_progress(
            progress_key,
            crate::backend::download_progress::DownloadProgress {
                url: m3u8_url.to_string(),
                filename: dest_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string(),
                total_size: 0,
                downloaded: 0,
                status: "erro".to_string(),
                id,
                scope: Some(scope.to_string()),
            }
        );
        return Err("Nenhum segmento .ts encontrado na playlist".to_string());
    }
    let mut file = if is_resuming {
        // Abre para leitura/escrita e posiciona no fim para retomar.
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&temp_dest)
            .map_err(|e| format!("Erro ao abrir arquivo para retomar: {}", e))?;
        f.seek(SeekFrom::Start(existing_temp_size))
            .map_err(|e| format!("Erro ao posicionar arquivo para retomar: {}", e))?;
        f
    } else {
        // Cria novo arquivo
        File::create(&temp_dest).map_err(|e| format!("Erro ao criar arquivo temporário: {}", e))?
    };

    // Restaura bytes_downloaded do progresso anterior se estiver retomando
    let mut bytes_downloaded = if is_resuming {
        existing_temp_size
    } else {
        0u64
    };
    
        let id_value = id.or_else(|| {
        dest_path.file_stem().and_then(|s| s.to_str()).and_then(|stem| stem.split('_').last().and_then(|n| n.parse::<u64>().ok()))
    }).unwrap_or(0);
    // Evento e log: preparando download (calculando tamanho total)
    if let Some(w) = window {
        let _ = w.emit("download-progress", serde_json::json!({
            "id": id_value,
            "progress": bytes_downloaded,
            "total": 0u64,
            "status": "preparando"
        }));
    }
    // Calcula o tamanho total real dos segmentos
    let mut total_size = if is_resuming {
        // Se está retomando, restaura o total_size anterior
        existing_progress.as_ref().map(|p| p.total_size).unwrap_or(0u64)
    } else {
        0u64
    };
    
    // Se não temos total_size ainda, calcula
    if total_size == 0 {
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
    }
    // Assim que o arquivo temporário é criado e o tamanho total é conhecido, emite 'baixando'
    if let Some(w) = window {
        let _ = w.emit("download-progress", serde_json::json!({
            "id": id_value,
            "progress": bytes_downloaded,
            "total": total_size,
            "status": "baixando"
        }));
    }
    crate::backend::download_progress::update_progress(
        progress_key,
        crate::backend::download_progress::DownloadProgress {
            url: m3u8_url.to_string(),
            filename: dest_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string(),
            total_size: total_size,
            downloaded: bytes_downloaded,
            status: "baixando".to_string(),
            id,
            scope: Some(scope.to_string()),
        }
    );

    // Em resume, pula segmentos já gravados no arquivo parcial.
    let mut remaining_skip_bytes = if is_resuming { existing_temp_size } else { 0 };

    // Durante o download dos segmentos, mantém status como "calculando" (ou "baixando segmentos" se desejar detalhar)
    for (i, seginfo) in segments.iter().enumerate() {
        // Checa flag global de pausa
        if pause_flag.load(std::sync::atomic::Ordering::SeqCst) {
            // Salva status pausado no progresso
            crate::backend::download_progress::update_progress(
                progress_key,
                crate::backend::download_progress::DownloadProgress {
                    url: m3u8_url.to_string(),
                    filename: dest_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string(),
                    total_size: total_size,
                    downloaded: bytes_downloaded,
                    status: "pausado".to_string(),
                    id: Some(id_value),
                    scope: Some(scope.to_string()),
                }
            );
            if let Some(w) = window {
                let _ = w.emit("download-progress", serde_json::json!({
                    "id": id_value,
                    "progress": bytes_downloaded,
                    "total": total_size,
                    "status": "pausado"
                }));
            }
            return Err("Download pausado pelo usuário".to_string());
        }

        if remaining_skip_bytes > 0 {
            let seg_size = client
                .head(&seginfo.url)
                .send()
                .ok()
                .and_then(|resp| {
                    resp.headers()
                        .get(reqwest::header::CONTENT_LENGTH)
                        .and_then(|len| len.to_str().ok())
                        .and_then(|s| s.parse::<u64>().ok())
                })
                .unwrap_or(0);

            if seg_size > 0 && remaining_skip_bytes >= seg_size {
                remaining_skip_bytes -= seg_size;
                continue;
            }

            if seg_size > 0 && remaining_skip_bytes > 0 {
                let aligned_size = existing_temp_size.saturating_sub(remaining_skip_bytes);
                file.set_len(aligned_size)
                    .map_err(|e| format!("Erro ao alinhar arquivo parcial: {}", e))?;
                file.seek(SeekFrom::Start(aligned_size))
                    .map_err(|e| format!("Erro ao reposicionar arquivo parcial: {}", e))?;
                bytes_downloaded = aligned_size;
                remaining_skip_bytes = 0;
            }
        }

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
        // Garante que bytes_downloaded nunca diminui (protege contra sobrescrita acidental)
        let novo_bytes_downloaded = bytes_downloaded.saturating_add(seg_len);
        if novo_bytes_downloaded >= bytes_downloaded {
            bytes_downloaded = novo_bytes_downloaded;
        } else {
        }
        // Emite progresso para o frontend com status 'baixando' (não mais 'calculando')
        let progress_json = serde_json::json!({
            "id": id_value,
            "progress": bytes_downloaded,
            "total": total_size,
            "status": "baixando"
        });
        if let Some(w) = window {
            let _ = w.emit("download-progress", progress_json.clone());
        }
        // Atualiza progresso persistente a cada segmento
        crate::backend::download_progress::update_progress(
            progress_key,
            crate::backend::download_progress::DownloadProgress {
                url: m3u8_url.to_string(),
                filename: dest_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string(),
                total_size: total_size,
                downloaded: bytes_downloaded,
                status: "baixando".to_string(),
                id: Some(id_value),
                scope: Some(scope.to_string()),
            }
        );
        
    }
    let meta = std::fs::metadata(&temp_dest).map_err(|e| format!("Erro ao finalizar arquivo temporário: {}", e))?;
    if meta.len() == 0 {
        return Err("Arquivo HLS criado mas está vazio".to_string());
    }

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
        progress_key,
        crate::backend::download_progress::DownloadProgress {
            url: m3u8_url.to_string(),
            filename: dest_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string(),
            total_size: total_size,
            downloaded: bytes_downloaded,
            status: "convertendo".to_string(),
            id: Some(id_value),
            scope: Some(scope.to_string()),
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
        let ffmpeg_status = Command::new("ffmpeg")
            .args(&["-y", "-i", temp_dest_clone.to_str().unwrap(), "-c", "copy", dest_path_for_thread.to_str().unwrap()])
            .status();
        let result = match ffmpeg_status {
            Ok(status) if status.success() => {
                let _ = std::fs::remove_file(&temp_dest_clone);
                Ok(())
            }
            Ok(status) => {
                Err(format!("FFmpeg falhou com código {}", status))
            }
            Err(e) => {
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
            // Agora sim, arquivo final foi criado: emite status 'baixando' brevemente, depois 'concluído'
            if let Some(w) = window {
                let _ = w.emit("download-progress", serde_json::json!({
                    "id": id_value_clone,
                    "progress": bytes_downloaded,
                    "total": bytes_downloaded,
                    "status": "baixando"
                }));
            }
            // Persiste status 'baixando' brevemente
            crate::backend::download_progress::update_progress(progress_key, crate::backend::download_progress::DownloadProgress {
                url: m3u8_url_clone.clone(),
                filename: dest_path_clone.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string(),
                total_size: bytes_downloaded,
                downloaded: bytes_downloaded,
                status: "baixando".to_string(),
                id: Some(id_value_clone),
                scope: Some(scope.to_string()),
            });
            // Pequeno delay para garantir que o frontend mostre 'baixando' (opcional, pode ajustar ou remover)
            std::thread::sleep(std::time::Duration::from_millis(400));
            let progress = crate::backend::download_progress::DownloadProgress {
                url: m3u8_url_clone.clone(),
                filename: dest_path_clone.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string(),
                total_size: bytes_downloaded,
                downloaded: bytes_downloaded,
                status: "concluído".to_string(),
                id: Some(id_value_clone),
                scope: Some(scope.to_string()),
            };
            crate::backend::download_progress::update_progress(progress_key, progress.clone());
            if let Some(w) = window {
                if scope == "home" {
                    if let Some(username) = username {
                        if let Err(_e) = crate::backend::user_service::update_main_url_progress(
                            username.to_string(),
                            m3u8_url_clone.clone(),
                            1.0,
                        ) {
                            // log removido
                        }
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


/// Gera um caminho de destino único: se `path` já existir, tenta
/// `stem(1).ext`, `stem(2).ext`, ... até encontrar um nome livre.
pub fn unique_save_path(path: &Path) -> std::path::PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("video");
    let ext  = path.extension().and_then(|e| e.to_str()).unwrap_or("mp4");
    let parent = path.parent().unwrap_or(Path::new("."));
    let mut counter = 1u32;
    loop {
        let candidate = parent.join(format!("{}({}).{}", stem, counter, ext));
        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
    }
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
    progress_key: Option<&str>,
    source: Option<&str>,
) -> Result<(), String> {
    let progress_key = progress_key.unwrap_or(m3u8_url);
    let scope = source.unwrap_or("home");
    // Depuração: início do download JMVStream

    // Se não há arquivo parcial (.ts) em andamento mas o destino final já existe,
    // Verifica arquivo parcial anterior com o nome original
    let initial_temp = { let mut t = dest_path.to_path_buf(); t.set_extension("ts"); t };
    let existing_temp_size_check = std::fs::metadata(&initial_temp).map(|m| m.len()).unwrap_or(0);

    // gera um nome único: video(1).mp4, video(2).mp4, etc.
    let final_dest: std::path::PathBuf = if existing_temp_size_check == 0 && dest_path.exists() {
        let u = unique_save_path(dest_path);
        u
    } else {
        dest_path.to_path_buf()
    };
    let dest_path = final_dest.as_path();
    let mut temp_dest = dest_path.to_path_buf();
    temp_dest.set_extension("ts");
    let existing_temp_size = std::fs::metadata(&temp_dest).map(|m| m.len()).unwrap_or(0);
    let existing_progress = crate::backend::download_progress::get_progress(progress_key);
    // Emite e salva status 'preparando' antes de calcular o tamanho
    let filename = dest_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
    if let Some(w) = window {
        let _ = w.emit("download-progress", serde_json::json!({
            "id": id.unwrap_or(0),
            "progress": existing_temp_size,
            "total": 0u64,
            "status": "preparando"
        }));
    }
    let progress_preparando = crate::backend::download_progress::DownloadProgress {
        url: m3u8_url.to_string(),
        filename: filename.clone(),
        total_size: 0,
        downloaded: existing_temp_size,
        status: "preparando".to_string(),
        id,
        scope: Some(scope.to_string()),
    };
    crate::backend::download_progress::update_progress(progress_key, progress_preparando);

    // Calcula o tamanho total antes de iniciar
    let total_size = existing_progress
        .as_ref()
        .map(|p| p.total_size)
        .filter(|s| *s > 0)
        .unwrap_or_else(|| calcular_tamanho_hls(client, m3u8_url).unwrap_or(0));

    // Só muda para 'baixando' quando realmente iniciar o download dos segmentos (dentro do fluxo de download)
    // O progresso salvo aqui ainda é 'preparando', mas já com o tamanho correto
    let progress = crate::backend::download_progress::DownloadProgress {
        url: m3u8_url.to_string(),
        filename: filename.clone(),
        total_size,
        downloaded: existing_temp_size,
        status: "preparando".to_string(),
        id,
        scope: Some(scope.to_string()),
    };
    crate::backend::download_progress::update_progress(progress_key, progress.clone());

    // Chama o fluxo padrão de download HLS, mas agora o progresso já tem o tamanho real
    download_hls_file(
        client,
        m3u8_url,
        dest_path,
        window,
        Some("download_progress"),
        id,
        username,
        should_stop,
        Some(progress_key),
        Some(scope),
    )
}



