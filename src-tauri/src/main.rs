#[derive(Serialize, Deserialize, Clone)]
struct User {
	username: String,
	password: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct UserList {
	users: Vec<User>,
}

#[command]
fn cadastrar_usuario(username: String, password: String) -> Result<(), String> {
	let path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join("src-tauri/src/user.json");
	// Garante que o diretório existe
	if let Some(parent) = path.parent() {
		std::fs::create_dir_all(parent).map_err(|e| format!("Erro ao criar diretório: {}", e))?;
	}
	let mut user_list: UserList = if let Ok(data) = std::fs::read_to_string(&path) {
		serde_json::from_str(&data).unwrap_or_default()
	} else {
		UserList::default()
	};
	if user_list.users.iter().any(|u| u.username == username) {
		return Err("Usuário já cadastrado".to_string());
	}
	user_list.users.push(User { username, password });
	let json = serde_json::to_string_pretty(&user_list).map_err(|e| e.to_string())?;
	std::fs::write(&path, json).map_err(|e| e.to_string())?;
	Ok(())
}

#[command]
fn autenticar_usuario(username: String, password: String) -> Result<(), String> {
	let path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join("src-tauri/src/user.json");
	// Garante que o diretório existe
	if let Some(parent) = path.parent() {
		std::fs::create_dir_all(parent).map_err(|e| format!("Erro ao criar diretório: {}", e))?;
	}
	let user_list: UserList = if let Ok(data) = std::fs::read_to_string(&path) {
		serde_json::from_str(&data).unwrap_or_default()
	} else {
		UserList::default()
	};
	if user_list.users.iter().any(|u| u.username == username && u.password == password) {
		Ok(())
	} else {
		Err("Usuário ou senha inválidos".to_string())
	}
}
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Clone)]
struct Playlist {
	title: String,
	downloaded: bool,
}

#[allow(dead_code)]
#[command]
fn marcar_playlist_baixada(title: String) -> Result<(), String> {
	let path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join("src-tauri/src/playlist.json");
	let mut playlists: Vec<Playlist> = if let Ok(data) = std::fs::read_to_string(&path) {
		serde_json::from_str(&data).unwrap_or_default()
	} else {
		Vec::new()
	};
	for pl in playlists.iter_mut() {
		if pl.title == title {
			pl.downloaded = true;
		}
	}
	let json = serde_json::to_string_pretty(&playlists).map_err(|e| e.to_string())?;
	std::fs::write(&path, json).map_err(|e| e.to_string())?;
	Ok(())
}

#[allow(dead_code)]
#[command]
fn listar_playlists() -> Vec<Playlist> {
	let path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join("src-tauri/src/playlist.json");
	if let Ok(data) = std::fs::read_to_string(&path) {
		serde_json::from_str(&data).unwrap_or_default()
	} else {
		Vec::new()
	}
}

#[allow(dead_code)]
#[command]
fn salvar_playlist(title: String) -> Result<(), String> {
	let path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join("src-tauri/src/playlist.json");
	let mut playlists: Vec<Playlist> = if let Ok(data) = std::fs::read_to_string(&path) {
		serde_json::from_str(&data).unwrap_or_default()
	} else {
		Vec::new()
	};
	if !playlists.iter().any(|pl| pl.title == title) {
		playlists.push(Playlist { title, downloaded: false });
	}
	let json = serde_json::to_string_pretty(&playlists).map_err(|e| e.to_string())?;
	std::fs::write(&path, json).map_err(|e| e.to_string())?;
	Ok(())
}
use tauri::command;
use tauri::Manager;
use tauri::Emitter;
use tauri::Window;
use std::fs;
#[command]
fn listar_videos_baixados() -> Vec<std::collections::HashMap<String, String>> {
	let mut result = Vec::new();
	let dir = std::env::current_dir().unwrap_or_default().parent().unwrap_or_else(|| std::path::Path::new("")).join("Vídeos baixados");
	if let Ok(entries) = fs::read_dir(dir) {
		for entry in entries.flatten() {
			let path = entry.path();
			if path.is_file() {
				let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
				let path_str = path.to_string_lossy().to_string();
				let mut map = std::collections::HashMap::new();
				map.insert("name".to_string(), name);
				map.insert("path".to_string(), path_str);
				result.push(map);
			}
		}
	}
	result
}

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use regex::Regex;

fn baixar_video(url: &str, filename: &str) -> Result<(), String> {
	println!("[CLI] Iniciando download: {}", url);
	let response = reqwest::blocking::get(url).map_err(|e| format!("Erro ao baixar: {}", e))?;
	if !response.status().is_success() {
		return Err(format!("HTTP {}: {}", response.status().as_u16(), response.status()));
	}
  let _total_size = response.content_length().unwrap_or(0);
	let ext = filename;
	let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
	if let Some(parent) = path.parent() {
		path = parent.to_path_buf();
	}
	path.push("Vídeos baixados");
	std::fs::create_dir_all(&path).map_err(|e| format!("Erro ao criar pasta: {}", e))?;
	path.push(ext);
	let mut file = File::create(&path).map_err(|e| format!("Erro ao criar arquivo: {}", e))?;
	let bytes = response.bytes().map_err(|e| format!("Erro ao ler bytes: {}", e))?;
	let downloaded = bytes.len() as u64;
	file.write_all(&bytes).map_err(|e| format!("Erro ao salvar: {}", e))?;
	println!("[CLI] Download finalizado: {:?} ({} bytes)", path, downloaded);
	Ok(())
}

fn baixar_hls_emit(window: &Window, m3u8_url: &str, filename: &str, id: Option<u64>) -> Result<(), String> {
	println!("[HLS] Baixando playlist: {}", m3u8_url);
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
	println!("[HLS] Conteúdo da playlist:\n{}", text);
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
		let (bw, url) = variant_playlists.iter().max_by_key(|(bw, _)| *bw).unwrap();
		println!("[HLS] Master playlist detectada. Selecionando sub-playlist de maior qualidade (bandwidth: {}) => {}", bw, url);
		return baixar_hls_emit(window, url, filename, id);
	}
	if ts_urls.is_empty() {
		return Err("Nenhum segmento .ts encontrado na playlist".to_string());
	}
	let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
	if let Some(parent) = path.parent() {
		path = parent.to_path_buf();
	}
	path.push("Vídeos baixados");
	std::fs::create_dir_all(&path).map_err(|e| format!("Erro ao criar pasta: {}", e))?;
	path.push(filename);
	if ts_urls.is_empty() {
		return Err("Nenhum segmento .ts válido encontrado na playlist".to_string());
	}
	let mut file = File::create(&path).map_err(|e| format!("Erro ao criar arquivo: {}", e))?;
	for (i, ts_url) in ts_urls.iter().enumerate() {
		println!("[HLS] Baixando segmento {}/{}", i+1, ts_urls.len());
		let seg = client.get(ts_url)
			.send().map_err(|e| format!("Erro ao baixar segmento: {}", e))?
			.bytes().map_err(|e| format!("Erro ao ler segmento: {}", e))?;
		file.write_all(&seg).map_err(|e| format!("Erro ao salvar segmento: {}", e))?;
		// Emitir progresso para o frontend
		let progress = ((i + 1) as f64 / ts_urls.len() as f64 * 100.0).round() as u8;
		let _ = window.emit("download_progress", match id {
			Some(id) => serde_json::json!({ "id": id, "progress": progress }),
			None => serde_json::json!({ "filename": filename, "progress": progress })
		});
	}
	println!("[HLS] Download finalizado: {:?}", path);
	Ok(())
}

fn baixar_player_jmvstream(window: &Window, player_url: &str, output: &str, id: Option<u64>) -> Result<(), String> {
	println!("[EXTRAÇÃO] Baixando HTML do player: {}", player_url);
	let client = reqwest::blocking::Client::builder()
		.user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
		.build()
		.map_err(|e| format!("Erro ao criar client: {}", e))?;
	let html = client.get(player_url)
		.header("Referer", player_url)
		.send().map_err(|e| format!("Erro ao baixar HTML do player: {}", e))?
		.text().map_err(|e| format!("Erro ao ler HTML do player: {}", e))?;
	println!("[EXTRAÇÃO] HTML baixado:\n{}", html);
	// Regex melhorado para pegar src, file, source, ou links .m3u8 em qualquer lugar
	let re = Regex::new(r#"(src|file|source)"\s*:\s*"([^"]+\.m3u8[^"]*)"|https?://[^"]+\.m3u8[^"]*"#).unwrap();
	let m3u8_url = if let Some(cap) = re.captures(&html) {
		if let Some(url) = cap.get(2) {
			Some(url.as_str())
		} else {
			// fallback para pegar o link completo
			let full = cap.get(0).map(|m| m.as_str());
			full.and_then(|s| s.split('"').find(|x| x.contains(".m3u8")))
		}
	} else {
		None
	};
	let m3u8_url = match m3u8_url {
		Some(url) => url,
		None => return Err("Não foi possível extrair o link .m3u8 do player".to_string()),
	};
	println!("[EXTRAÇÃO] Link .m3u8 extraído: {}", m3u8_url);
	baixar_hls_emit(window, m3u8_url, output, id)
}

#[command]
fn baixar_video_tauri(window: Window, url: String, filename: String, id: Option<u64>) -> Result<(), String> {
	if url.ends_with(".m3u8") || url.contains(".m3u8?") {
		baixar_hls_emit(&window, &url, &filename, id)
	} else if url.contains("player.jmvstream.com") {
		baixar_player_jmvstream(&window, &url, &filename, id)
	} else {
		baixar_video(&url, &filename)
	}
}

fn main() {
	tauri::Builder::default()
		.setup(|app| {
			// Obtém a janela principal corretamente no Tauri v2
			if let Some(window) = app.get_webview_window("main") {
				if let Ok(Some(monitor)) = window.primary_monitor() {
					let size = monitor.size();
					let width = size.width / 2;
					let height = size.height;
					window.set_size(tauri::Size::Physical(tauri::PhysicalSize { width, height })).unwrap();
					window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x: 0, y: 0 })).unwrap();
				}
			}
			Ok(())
		})
		.invoke_handler(tauri::generate_handler![baixar_video_tauri, listar_videos_baixados, cadastrar_usuario, autenticar_usuario])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}

