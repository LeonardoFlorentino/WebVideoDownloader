// Função para uso CLI (sem janela Tauri)
pub async fn baixar_video_cli(url: String, filename: Option<String>) -> Result<String, String> {
  use std::path::PathBuf;
  use std::fs::File;
  use std::io::Write;
  println!("[CLI] Iniciando download: {}", url);
  let response = match reqwest::get(&url).await {
    Ok(resp) => resp,
    Err(e) => {
      println!("[CLI] Erro ao baixar: {}", e);
      return Err(format!("Erro ao baixar: {}", e));
    }
  };
  println!("[CLI] Status HTTP: {}", response.status());
  if !response.status().is_success() {
    println!("[CLI] HTTP ERROR: {} {}", response.status().as_u16(), response.status());
    return Err(format!("HTTP {}: {}", response.status().as_u16(), response.status()));
  }
  let total_size = response.content_length().unwrap_or(0);
  println!("[CLI] Content-Length: {}", total_size);
  let ext = filename.as_deref().unwrap_or("video.mp4");
  let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
  path.push("Vídeos baixados");
  if let Err(e) = std::fs::create_dir_all(&path) {
    println!("[CLI] Erro ao criar pasta: {}", e);
    return Err(format!("Erro ao criar pasta: {}", e));
  }
  path.push(ext);
  let mut file = match File::create(&path) {
    Ok(f) => f,
    Err(e) => {
      println!("[CLI] Erro ao criar arquivo: {}", e);
      return Err(format!("Erro ao criar arquivo: {}", e));
    }
  };

  let mut downloaded: u64 = 0;
  let mut stream = response.bytes_stream();
  use futures_util::StreamExt;
  while let Some(item) = stream.next().await {
    match item {
      Ok(chunk) => {
        if let Err(e) = file.write_all(&chunk) {
          println!("[CLI] Erro ao salvar chunk: {}", e);
          return Err(format!("Erro ao salvar: {}", e));
        }
        downloaded += chunk.len() as u64;
        let percent = if total_size > 0 {
          (downloaded as f64 / total_size as f64 * 100.0).round() as u8
        } else {
          0
        };
        println!("[CLI] Progresso: {}%", percent);
      },
      Err(e) => {
        println!("[CLI] Erro ao baixar chunk: {}", e);
        return Err(format!("Erro ao baixar chunk: {}", e));
      }
    }
  }
  // Verifica se o arquivo existe e tem tamanho > 0
  match std::fs::metadata(&path) {
    Ok(meta) => {
      println!("[CLI] Arquivo salvo: {:?} ({} bytes)", path, meta.len());
      if meta.len() == 0 {
        println!("[CLI] Arquivo baixado está vazio!");
        return Err("Arquivo baixado está vazio".to_string());
      }
    },
    Err(e) => {
      println!("[CLI] Erro ao verificar arquivo: {}", e);
      return Err("Arquivo não foi salvo corretamente".to_string());
    }
  }
  println!("[CLI] Download finalizado com sucesso: {:?}", path);
  Ok(path.to_string_lossy().to_string())
}
use tauri::Emitter;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[tauri::command]
async fn baixar_video(window: tauri::Window, url: String, filename: Option<String>) -> Result<String, String> {
  println!("[TAURI] Iniciando download: {}", url);
  let response = match reqwest::get(&url).await {
    Ok(resp) => resp,
    Err(e) => {
      println!("[TAURI] Erro ao baixar: {}", e);
      return Err(format!("Erro ao baixar: {}", e));
    }
  };
  println!("[TAURI] Status HTTP: {}", response.status());
  if !response.status().is_success() {
    println!("[TAURI] HTTP ERROR: {} {}", response.status().as_u16(), response.status());
    return Err(format!("HTTP {}: {}", response.status().as_u16(), response.status()));
  }
  let total_size = response.content_length().unwrap_or(0);
  println!("[TAURI] Content-Length: {}", total_size);
  let ext = filename.as_deref().unwrap_or("video.mp4");
  let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
  path.push("Vídeos baixados");
  if let Err(e) = std::fs::create_dir_all(&path) {
    println!("[TAURI] Erro ao criar pasta: {}", e);
    return Err(format!("Erro ao criar pasta: {}", e));
  }
  path.push(ext);
  let mut file = match File::create(&path) {
    Ok(f) => f,
    Err(e) => {
      println!("[TAURI] Erro ao criar arquivo: {}", e);
      return Err(format!("Erro ao criar arquivo: {}", e));
    }
  };

  let mut downloaded: u64 = 0;
  let mut stream = response.bytes_stream();
  use futures_util::StreamExt;
  while let Some(item) = stream.next().await {
    match item {
      Ok(chunk) => {
        if let Err(e) = file.write_all(&chunk) {
          println!("[TAURI] Erro ao salvar chunk: {}", e);
          return Err(format!("Erro ao salvar: {}", e));
        }
        downloaded += chunk.len() as u64;
        let percent = if total_size > 0 {
          (downloaded as f64 / total_size as f64 * 100.0).round() as u8
        } else {
          0
        };
        let _ = window.emit("download_progress", serde_json::json!({
          "filename": ext,
          "progress": percent
        }));
      },
      Err(e) => {
        println!("[TAURI] Erro ao baixar chunk: {}", e);
        return Err(format!("Erro ao baixar chunk: {}", e));
      }
    }
  }
  // Verifica se o arquivo existe e tem tamanho > 0
  match std::fs::metadata(&path) {
    Ok(meta) => {
      println!("[TAURI] Arquivo salvo: {:?} ({} bytes)", path, meta.len());
      if meta.len() == 0 {
        println!("[TAURI] Arquivo baixado está vazio!");
        return Err("Arquivo baixado está vazio".to_string());
      }
    },
    Err(e) => {
      println!("[TAURI] Erro ao verificar arquivo: {}", e);
      return Err("Arquivo não foi salvo corretamente".to_string());
    }
  }
  println!("[TAURI] Download finalizado com sucesso: {:?}", path);
  Ok(path.to_string_lossy().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![baixar_video])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
