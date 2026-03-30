use std::io::Write;
use std::fs::File;
use std::sync::{Arc, atomic::AtomicBool};
use tauri::{Window, Emitter};
use crate::backend::download_progress::{DownloadProgress, update_progress};
use bytes;

/// Escreve os chunks do download, atualiza progresso e emite eventos.
/// Retorna Ok(()) se download completo, ou Err se pausado/erro.
pub async fn write_download_chunks<St>(
    mut stream: St,
    mut file: File,
    mut progress: DownloadProgress,
    url: &str,
    total_size: u64,
    should_stop: Arc<AtomicBool>,
    window: Option<&Window>,
) -> Result<(), String>
where
    St: futures_util::stream::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
{
    use futures_util::StreamExt;
    use tokio::time::{timeout, Duration};
    let mut current = progress.downloaded;
    loop {
        if should_stop.load(std::sync::atomic::Ordering::SeqCst) {
            progress.status = "pausado".to_string();
            update_progress(url, progress.clone());
            if let Some(w) = window {
                let _ = w.emit("download_paused", serde_json::json!({ "url": url }));
            }
            return Err("Download pausado".to_string());
        }
        let next_chunk = timeout(Duration::from_secs(2), stream.next()).await;
        match next_chunk {
            Ok(Some(chunk_result)) => {
                let chunk = chunk_result.map_err(|e| format!("Erro ao ler chunk: {}", e))?;
                if chunk.is_empty() {
                    break;
                }
                file.write_all(&chunk).map_err(|e| format!("Erro ao salvar: {}", e))?;
                current += chunk.len() as u64;
                progress.downloaded = current;
                update_progress(url, progress.clone());
                if let Some(w) = window {
                    let _ = w.emit("download-progress", serde_json::json!({
                        "id": progress.id.unwrap_or(0).to_string(),
                        "progress": current,
                        "total": total_size,
                        "status": "baixando"
                    }));
                }
            }
            Ok(None) => {
                break;
            }
            Err(_elapsed) => {
                if should_stop.load(std::sync::atomic::Ordering::SeqCst) {
                    progress.status = "pausado".to_string();
                    update_progress(url, progress.clone());
                    if let Some(w) = window {
                        let _ = w.emit("download_paused", serde_json::json!({ "url": url }));
                    }
                    return Err("Download pausado".to_string());
                }
                continue;
            }
        }
    }
    Ok(())
}
