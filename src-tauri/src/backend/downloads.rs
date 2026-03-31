use crate::backend::download_chunk_writer::write_download_chunks;
use reqwest::header::{RANGE, CONTENT_LENGTH};
use crate::backend::http_client_helper::create_async_client;
use std::sync::{Arc, atomic::AtomicBool};
use crate::backend::download_pause_monitor::start_pause_monitor;
use crate::backend::download_progress::{DownloadProgress, update_progress, remove_progress};
use tokio::runtime::Runtime;
use crate::backend::download_helpers::{open_file_append, seek_file_end};
// use std::path::PathBuf;

pub fn baixar_video_emit(window: Option<&Window>, url: &str, _filename: &str) -> Result<(), String> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let _now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        // log removido
    let should_stop = Arc::new(AtomicBool::new(false));
    let url_string = url.to_string();
    // Ao iniciar novo download, só defina status 'baixando' se NÃO estiver pausado
    let mut path = crate::backend::filesystem::get_project_root();
    path.push("Vídeos baixados");
    let ext = url.split('/').last().unwrap_or("video.mp4").to_string();
    path.push(&ext);
    let file_empty = !path.exists() || std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0) == 0;
    if file_empty {
        // Se arquivo está vazio, remove progresso antigo para garantir reinício limpo
        crate::backend::download_progress::remove_progress(&url_string);
        // log removido
    }
    if let Some(mut prog) = crate::backend::download_progress::get_progress(&url_string) {
        // log removido
        if prog.status == "pausado" && !file_empty {
            // log removido
            return Ok(());
        } else {
            prog.status = "baixando".to_string();
            crate::backend::download_progress::update_progress(&url_string, prog);
            // log removido
        }
    } else {
        // log removido
    }
    // Usar helper para monitorar status de pausa
    start_pause_monitor(url_string.clone(), should_stop.clone());

    // Cria runtime tokio local para rodar async
    let rt = Runtime::new().map_err(|e| format!("Erro ao criar runtime: {}", e))?;
    rt.block_on(async {
        let client = create_async_client()?;

        // Sempre usa o nome real do arquivo extraído da URL
        let ext = url.split('/').last().unwrap_or("video.mp4").to_string();
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
        // log removido
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
                // log removido
                return Ok(());
            }
            latest.filename = ext.clone();
            latest.total_size = total_size;
            // Sempre sincroniza o progresso salvo com o arquivo parcial no disco
            let disk_downloaded = if path.exists() {
                std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
            } else {
                0
            };
            if disk_downloaded > latest.downloaded {
                // log removido
                latest.downloaded = disk_downloaded;
            }
            latest.status = "baixando".to_string();
            // Garante que o id seja mantido
            latest
        } else {
            // Tenta extrair id do nome do arquivo, se possível
            let id = if let Some(stem) = std::path::Path::new(&ext).file_stem().and_then(|s| s.to_str()) {
                stem.split('_').last().and_then(|n| n.parse::<u64>().ok())
            } else {
                None
            };
            let disk_downloaded = if path.exists() {
                std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
            } else {
                0
            };
            DownloadProgress {
                url: url.to_string(),
                filename: ext.clone(),
                total_size,
                downloaded: disk_downloaded,
                status: "baixando".to_string(),
                id,
            }
        };
        // log removido
        update_progress(url, progress.clone());

        // Abre arquivo para append
        let mut file = open_file_append(&path)?;

        // Garante que o ponteiro está no fim
        seek_file_end(&mut file, downloaded)?;

        // Baixa em chunks
        let mut stream = resp.bytes_stream();
        use futures_util::StreamExt;
        if let Some(first) = stream.next().await {
            match &first {
                Ok(chunk) => {
                    // log removido
                    use std::io::Write;
                    file.write_all(&chunk).map_err(|e| format!("Erro ao salvar primeiro chunk: {}", e))?;
                    progress.downloaded += chunk.len() as u64;
                    update_progress(url, progress.clone());
                },
                Err(e) => {
                    // log removido
                    return Err(format!("Erro ao receber primeiro chunk: {}", e));
                }
            }
        } else {
            // log removido
            return Err("Nenhum dado recebido do servidor".to_string());
        }
        // Continua normalmente com o restante do stream
        write_download_chunks(stream, file, progress.clone(), url, total_size, should_stop.clone(), window).await?;

        // Verifica se arquivo está vazio e finaliza download
        let meta = std::fs::metadata(&path).map_err(|e| format!("Erro ao finalizar arquivo: {}", e))?;
        if meta.len() == 0 {
            // log removido
            progress.status = "erro".to_string();
            update_progress(url, progress.clone());
            return Err("Arquivo criado mas está vazio".to_string());
        }

        // Marca como concluído
        if progress.status != "pausado" && progress.status != "erro" {
            progress.status = "concluído".to_string();
            progress.downloaded = total_size;
            // log removido
            // log removido
            update_progress(url, progress.clone());
            remove_progress(url); // Limpa progresso persistente ao concluir
        } else {
            // log removido
        }

        Ok(())
    })
}
use tauri::Window;



