use crate::backend::download_chunk_writer::write_download_chunks;
use reqwest::header::{RANGE, CONTENT_LENGTH};
use crate::backend::http_client_helper::{create_async_client, create_blocking_client};
use std::sync::{Arc, atomic::AtomicBool};
use crate::backend::download_pause_monitor::start_pause_monitor;
use crate::backend::download_progress::{DownloadProgress, update_progress, remove_progress};
use tokio::runtime::Runtime;
use crate::backend::download_helpers::{open_file_append, seek_file_end};
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
    // Usar helper para monitorar status de pausa
    start_pause_monitor(url_string.clone(), should_stop.clone());

    // Cria runtime tokio local para rodar async
    let rt = Runtime::new().map_err(|e| format!("Erro ao criar runtime: {}", e))?;
    rt.block_on(async {
        let client = create_async_client()?;

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
        let mut file = open_file_append(&path)?;

        // Garante que o ponteiro está no fim
        seek_file_end(&mut file, downloaded)?;

        // Baixa em chunks
        let stream = resp.bytes_stream();
        // Modularizar escrita de chunks e progresso
        write_download_chunks(stream, file, progress.clone(), url, total_size, should_stop.clone(), window).await?;

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
use tauri::Window;
use regex::Regex;


pub fn baixar_hls_emit(window: &Window, m3u8_url: &str, filename: &str, _id: Option<u64>) -> Result<(), String> {
    let client = create_blocking_client()?;
    let mut path = crate::backend::filesystem::get_project_root();
    path.push("Vídeos baixados");
    std::fs::create_dir_all(&path).map_err(|e| format!("Erro ao criar pasta: {}", e))?;
    path.push(filename);
    crate::backend::download_helpers::download_hls_file(&client, m3u8_url, &path, Some(window), Some("download_progress"))
}

pub fn baixar_player_jmvstream(window: &Window, username: &str, player_url: &str, output: &str, _id: Option<u64>) -> Result<(), String> {
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
    let mut dest = crate::backend::filesystem::get_project_root();
    dest.push("Vídeos baixados");
    std::fs::create_dir_all(&dest).map_err(|e| format!("Erro ao criar pasta: {}", e))?;
    dest.push(output);
    crate::backend::download_helpers::download_hls_file(&client, m3u8_url, &dest, Some(window), Some("download_progress"))
}

