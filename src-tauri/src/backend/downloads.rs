use reqwest::header::{RANGE, CONTENT_LENGTH};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use crate::backend::download_progress::{DownloadProgress, update_progress, remove_progress};
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use tokio::runtime::Runtime;
// use std::path::PathBuf;

pub fn baixar_video_emit(window: Option<&Window>, url: &str, filename: &str) -> Result<(), String> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        println!("[DOWNLOAD ENTRY] baixar_video_emit chamada para URL: {} em {}", url, now);
    let should_stop = Arc::new(AtomicBool::new(false));
    let url_string = url.to_string();
    // Ao iniciar novo download, só defina status 'baixando' se NÃO estiver pausado
    if let Some(mut prog) = crate::backend::download_progress::get_progress(&url_string) {
        println!("[DOWNLOAD INIT] Status inicial para {}: {}", url_string, prog.status);
        if prog.status == "pausado" {
            println!("[DOWNLOAD INIT] Status já está 'pausado' para {}, não sobrescrevendo.", url_string);
            return Ok(()); // Não inicia o download se já estiver pausado
        } else {
            prog.status = "baixando".to_string();
            crate::backend::download_progress::update_progress(&url_string, prog);
            println!("[DOWNLOAD INIT] Status setado para 'baixando' para {}", url_string);
        }
    } else {
        println!("[DOWNLOAD INIT] Nenhum progresso anterior encontrado para {}", url_string);
    }
    let should_stop_clone = should_stop.clone();
    // Thread para monitorar status de pausa
    thread::spawn(move || {
        use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
        static MONITOR_COUNTER: once_cell::sync::Lazy<AtomicUsize> = once_cell::sync::Lazy::new(|| AtomicUsize::new(0));
        let mut last_status = String::new();
        loop {
            std::thread::sleep(std::time::Duration::from_millis(300));
            let count = MONITOR_COUNTER.fetch_add(1, AtomicOrdering::SeqCst);
            if let Some(prog) = crate::backend::download_progress::get_progress(&url_string) {
                if prog.status != last_status {
                    println!("[PAUSE MONITOR] status for {}: {}", url_string, prog.status);
                    last_status = prog.status.clone();
                } else if count % 10 == 0 {
                    println!("[PAUSE MONITOR] status for {}: {} (periodic)", url_string, prog.status);
                }
                if prog.status == "pausado" {
                    println!("[PAUSE MONITOR] Detected pause for {}!", url_string);
                    should_stop_clone.store(true, Ordering::SeqCst);
                    break;
                }
            } else if count % 10 == 0 {
                println!("[PAUSE MONITOR] No progress found for {} (periodic)", url_string);
            }
        }
    });

    // Cria runtime tokio local para rodar async
    let rt = Runtime::new().map_err(|e| format!("Erro ao criar runtime: {}", e))?;
    rt.block_on(async {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0")
            .build()
            .map_err(|e| format!("Erro ao criar client: {}", e))?;

        // Garante que o nome termina exatamente em .mp4, cortando qualquer coisa após .mp4
        let ext = if let Some(idx) = filename.find(".mp4") {
            let end = idx + 4;
            filename[..end].to_string()
        } else {
            format!("{}.mp4", filename)
        };

        let mut path = crate::backend::filesystem::get_project_root();
        path.push("Vídeos baixados");
        std::fs::create_dir_all(&path).map_err(|e| format!("Erro ao criar pasta: {}", e))?;
        path.push(&ext);

        // Verifica tamanho já baixado
        let mut downloaded: u64 = 0;
        if path.exists() {
            downloaded = std::fs::metadata(&path).map_err(|e| format!("Erro ao ler arquivo parcial: {}", e))?.len();
        }

        // Faz requisição com Range para obter tamanho total
        let mut req = client.get(url);
        if downloaded > 0 {
            req = req.header(RANGE, format!("bytes={}-", downloaded));
        }
        let resp = req.send().await.map_err(|e| format!("Erro ao baixar: {}", e))?;
        if !resp.status().is_success() && resp.status().as_u16() != 206 {
            return Err(format!("HTTP {}: {}", resp.status().as_u16(), resp.status()));
        }

        // Tamanho total esperado
        let total_size = if let Some(len) = resp.headers().get(CONTENT_LENGTH) {
            len.to_str().unwrap_or("0").parse::<u64>().unwrap_or(0) + downloaded
        } else {
            downloaded
        };

        // Sempre recarrega progresso do disco antes de inicializar progress
        let mut progress = if let Some(mut latest) = crate::backend::download_progress::get_progress(url) {
            if latest.status == "pausado" {
                println!("[DOWNLOAD] Progresso já está pausado ao iniciar, abortando.");
                return Ok(());
            }
            latest.filename = ext.clone();
            latest.total_size = total_size;
            latest.downloaded = downloaded;
            latest.status = "baixando".to_string();
            latest
        } else {
            DownloadProgress {
                url: url.to_string(),
                filename: ext.clone(),
                total_size,
                downloaded,
                status: "baixando".to_string(),
            }
        };
        println!("[DOWNLOAD] Atualizando progresso para status: {} ({} bytes)", progress.status, progress.downloaded);
        update_progress(url, progress.clone());

        // Abre arquivo para append
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| format!("Erro ao abrir arquivo: {}", e))?;

        // Garante que o ponteiro está no fim
        file.seek(SeekFrom::Start(downloaded)).map_err(|e| format!("Erro ao posicionar arquivo: {}", e))?;

        // Baixa em chunks
        let mut stream = resp.bytes_stream();
        let mut current = downloaded;
        use futures_util::StreamExt;
        use tokio::time::{timeout, Duration};
        // let mut completed = false;
        loop {
            // Sempre recarrega progresso do disco antes de qualquer update
            if should_stop.load(Ordering::SeqCst) {
                println!("[DOWNLOAD] Pausa detectada para {}! Interrompendo download.", url);
                progress.status = "pausado".to_string();
                update_progress(url, progress.clone());
                if let Some(w) = window {
                    println!("[DOWNLOAD] Emitindo evento download_paused para {}", url);
                    let _ = w.emit("download_paused", serde_json::json!({ "url": url }));
                }
                return Ok(());
            }
            // Recarrega status do progresso do disco ANTES de atualizar progresso
            if let Some(latest) = crate::backend::download_progress::get_progress(url) {
                if latest.status == "pausado" {
                    println!("[DOWNLOAD] Pausa detectada via status persistido para {}! Interrompendo download.", url);
                    // NÃO sobrescreve status, apenas retorna
                    if let Some(w) = window {
                        println!("[DOWNLOAD] Emitindo evento download_paused para {} (persist)", url);
                        let _ = w.emit("download_paused", serde_json::json!({ "url": url }));
                    }
                    return Ok(());
                }
            }
            let next_chunk = timeout(Duration::from_secs(2), stream.next()).await;
            match next_chunk {
                Ok(Some(chunk_result)) => {
                    let chunk = chunk_result.map_err(|e| format!("Erro ao ler chunk: {}", e))?;
                    if chunk.is_empty() {
                        println!("[DOWNLOAD] Fim do stream para {}", url);
                        break;
                    }
                    // Recarrega status do progresso do disco ANTES de atualizar progresso
                    if let Some(latest) = crate::backend::download_progress::get_progress(url) {
                        if latest.status == "pausado" {
                            println!("[DOWNLOAD] Detected 'pausado' antes de update_progress, interrompendo.");
                            // NÃO sobrescreve status, apenas retorna
                            if let Some(w) = window {
                                println!("[DOWNLOAD] Emitindo evento download_paused para {} (persist2)", url);
                                let _ = w.emit("download_paused", serde_json::json!({ "url": url }));
                            }
                            return Ok(());
                        }
                    }
                    file.write_all(&chunk).map_err(|e| format!("Erro ao salvar: {}", e))?;
                    current += chunk.len() as u64;
                    progress.downloaded = current;
                    update_progress(url, progress.clone());
                    let percent = if total_size > 0 {
                        (current as f64 / total_size as f64) * 100.0
                    } else {
                        0.0
                    };
                    if let Some(w) = window {
                        let _ = w.emit("download_progress", serde_json::json!({ "url": url, "progress": percent }));
                    }
                }
                Ok(None) => {
                    println!("[DOWNLOAD] Fim do stream para {} (Ok(None))", url);
                    break;
                }
                Err(_elapsed) => {
                    println!("[DOWNLOAD] Timeout aguardando chunk para {}", url);
                    if should_stop.load(Ordering::SeqCst) {
                        println!("[DOWNLOAD] Pausa detectada durante timeout para {}! Interrompendo download.", url);
                        progress.status = "pausado".to_string();
                        update_progress(url, progress.clone());
                        if let Some(w) = window {
                            println!("[DOWNLOAD] Emitindo evento download_paused para {} (timeout)", url);
                            let _ = w.emit("download_paused", serde_json::json!({ "url": url }));
                        }
                        return Ok(());
                    }
                    continue;
                }
            }
        }

        // Verifica se arquivo está vazio e finaliza download
        let meta = std::fs::metadata(&path).map_err(|e| format!("Erro ao finalizar arquivo: {}", e))?;
        if meta.len() == 0 {
            progress.status = "erro".to_string();
            update_progress(url, progress.clone());
            return Err("Arquivo criado mas está vazio".to_string());
        }

        // Marca como concluído
        if progress.status != "pausado" && progress.status != "erro" {
            progress.status = "concluido".to_string();
            progress.downloaded = total_size;
            update_progress(url, progress.clone());
            remove_progress(url); // Limpa progresso persistente ao concluir
        }

        Ok(())
    })
}
use tauri::Emitter;
use tauri::Window;
use regex::Regex;


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
