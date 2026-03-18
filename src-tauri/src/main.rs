#[derive(Serialize, Deserialize, Clone)]
struct MainUrl {
	url: String,
	filename: String,
}
// Atualiza o título/filename de uma URL em main_urls para um usuário
#[command]
fn update_main_url_title(username: String, old_url: String, new_url: String, new_filename: String) -> Result<(), String> {
	println!("[LOG] INÍCIO update_main_url_title: username='{}', old_url='{}', new_url='{}', new_filename='{}'", username, old_url, new_url, new_filename);
	let path = get_project_root().join("user_data/user.json");
	println!("[LOG] Caminho do user.json: {}", path.display());
	let user_list_result = std::fs::read_to_string(&path);
	match &user_list_result {
		Ok(data) => println!("[LOG] user.json lido com sucesso ({} bytes)", data.len()),
		Err(e) => println!("[ERRO] Falha ao ler user.json: {}", e),
	}
	let mut user_list: UserList = match user_list_result {
		Ok(data) => match serde_json::from_str(&data) {
			Ok(list) => {
				println!("[LOG] user.json desserializado com sucesso");
				list
			},
			Err(e) => {
				println!("[ERRO] Falha ao desserializar user.json: {}", e);
				return Err(format!("Erro ao desserializar user.json: {}", e));
			}
		},
		Err(e) => {
			println!("[ERRO] user.json não encontrado ou não pôde ser lido: {}", e);
			return Err(format!("Erro ao ler user.json: {}", e));
		}
	};
	let user_opt = user_list.users.iter_mut().find(|u| u.username == username);
	if user_opt.is_none() {
		println!("[ERRO] Usuário '{}' não encontrado", username);
		return Err("Usuário não encontrado".to_string());
	}
	let user = user_opt.unwrap();
	println!("[LOG] Usuário encontrado: {}", username);
	let main_url_opt = user.main_urls.iter_mut().find(|mu| mu.url == old_url);
	if main_url_opt.is_none() {
		println!("[ERRO] URL '{}' não encontrada para o usuário '{}'", old_url, username);
		return Err("URL não encontrada para o usuário".to_string());
	}
	let main_url = main_url_opt.unwrap();
	println!("[LOG] URL encontrada para edição: {}", old_url);
	main_url.url = new_url.clone();
	main_url.filename = new_filename.clone();
	let json_result = serde_json::to_string_pretty(&user_list);
	match &json_result {
		Ok(_) => println!("[LOG] Serialização do user_list bem-sucedida"),
		Err(e) => {
			println!("[ERRO] Falha ao serializar user_list: {}", e);
			return Err(format!("Erro ao serializar user_list: {}", e));
		}
	}
	let json = json_result.unwrap();
	let write_result = std::fs::write(&path, &json);
	match &write_result {
		Ok(_) => println!("[LOG] user.json salvo com sucesso"),
		Err(e) => {
			println!("[ERRO] Falha ao salvar user.json: {}", e);
			return Err(format!("Erro ao salvar user.json: {}", e));
		}
	}
	println!("[LOG] FIM update_main_url_title: sucesso para username='{}'", username);
	Ok(())
}
fn get_project_root() -> PathBuf {
	let mut path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
	loop {
		if path.join("package.json").exists() {
			return path;
		}
		if !path.pop() {
			break;
		}
	}
	std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
use tauri::Window;

#[command]
fn open_download_folder(playlist: String) -> Result<(), String> {
	use std::process::Command;
	let mut path = get_project_root();
	path.push("Vídeos baixados");
	path.push(&playlist);
	if path.exists() {
		#[cfg(target_os = "windows")]
		{
			Command::new("explorer").arg(path).spawn().map_err(|e| e.to_string())?;
		}
		#[cfg(target_os = "linux")]
		{
			Command::new("xdg-open").arg(path).spawn().map_err(|e| e.to_string())?;
		}
		#[cfg(target_os = "macos")]
		{
			Command::new("open").arg(path).spawn().map_err(|e| e.to_string())?;
		}
		Ok(())
	} else {
		Err("Pasta não encontrada".to_string())
	}
}
#[command]
fn get_main_urls(username: String) -> Result<Vec<MainUrl>, String> {
	let path = get_project_root().join("user_data/user.json");
	let user_list: UserList = if let Ok(data) = std::fs::read_to_string(&path) {
		serde_json::from_str(&data).unwrap_or_default()
	} else {
		UserList::default()
	};
	if let Some(user) = user_list.users.iter().find(|u| u.username == username) {
		Ok(user.main_urls.clone())
	} else {
		Err("Usuário não encontrado".to_string())
	}
}
#[command]
fn add_main_url(username: String, url: String, filename: Option<String>) -> Result<(), String> {
	let path = get_project_root().join("user_data/user.json");
	let mut user_list: UserList = if let Ok(data) = std::fs::read_to_string(&path) {
		serde_json::from_str(&data).unwrap_or_default()
	} else {
		UserList::default()
	};
	if let Some(user) = user_list.users.iter_mut().find(|u| u.username == username) {
		if !user.main_urls.iter().any(|mu| mu.url == url) {
			user.main_urls.push(MainUrl {
				url: url.clone(),
				filename: filename.unwrap_or_else(|| "video".to_string()),
			});
		}
		let json = serde_json::to_string_pretty(&user_list).map_err(|e| e.to_string())?;
		std::fs::write(&path, json).map_err(|e| e.to_string())?;
		Ok(())
	} else {
		Err("Usuário não encontrado".to_string())
	}
}
#[command]
fn baixar_em_cascata(window: Window, playlist: String, urls: Vec<String>) -> Result<(), String> {
	let pasta = playlist.clone();
	let title_re = Regex::new(r#"<title>(.*?)</title>"#).unwrap();
	let og_title_re = Regex::new(r#"<meta[^>]+property=["']og:title["'][^>]+content=["']([^"']+)["']"#).unwrap();
	for url in urls.iter() {
		println!("[DEBUG] Tentando baixar URL: {} para playlist: {}", url, playlist);
		// Baixar HTML da página
		let html = match reqwest::blocking::get(url)
			.and_then(|r| r.text()) {
			Ok(h) => h,
			Err(e) => {
				println!("[ERRO] Falha ao baixar HTML da URL {}: {}", url, e);
				return Err(format!("Erro ao baixar HTML: {}", e));
			}
		};
		// Extrair og:title ou <title>
		let title = if let Some(cap) = og_title_re.captures(&html) {
			cap.get(1).map(|m| m.as_str()).unwrap_or("video")
		} else if let Some(cap) = title_re.captures(&html) {
			cap.get(1).map(|m| m.as_str()).unwrap_or("video")
		} else {
			"video"
		};
		let sanitized = title.replace(|c: char| r#"<>:"/|?*"#.contains(c), "_");
		let filename = format!("{}.mp4", sanitized);
		// Caminho absoluto para a raiz do projeto
		let mut path = get_project_root();
		path.push("Vídeos baixados");
		path.push(&pasta);
		std::fs::create_dir_all(&path).map_err(|e| format!("Erro ao criar pasta: {}", e))?;
		path.push(&filename);

		let download_result = if url.contains("player.jmvstream.com") {
			baixar_player_jmvstream(&window, url, path.to_str().unwrap_or("video.mp4"), None)
		} else if url.ends_with(".m3u8") || url.contains(".m3u8?") {
			baixar_hls_emit(&window, url, path.to_str().unwrap_or("video.mp4"), None)
		} else {
			baixar_video(url, path.to_str().unwrap_or("video.mp4"))
		};
		if let Err(e) = download_result {
			println!("[ERRO] Falha ao baixar {}: {}", url, e);
			return Err(format!("Erro ao baixar {}: {}", url, e));
		}
		// Só retorna sucesso se o arquivo realmente existir e tiver tamanho > 0
		match std::fs::metadata(&path) {
			Ok(meta) => {
				if meta.len() == 0 {
					println!("[ERRO] Arquivo criado mas está vazio: {:?}", path);
					return Err("Arquivo criado mas está vazio".to_string());
				}
			}
			Err(e) => {
				println!("[ERRO] Não foi possível obter metadata do arquivo: {:?} => {}", path, e);
				return Err(format!("Arquivo não criado corretamente: {}", e));
			}
		}
		println!("[DEBUG] Download concluído para {}", url);
	}
	Ok(())
}
#[derive(Serialize, Deserialize, Clone)]
struct User {
	username: String,
	password: String,
	playlists: Vec<String>,
	main_urls: Vec<MainUrl>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct UserList {
	users: Vec<User>,
}


#[command]
fn cadastrar_usuario(username: String, password: String) -> Result<(), String> {
	let path = get_project_root().join("user_data/user.json");
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
	user_list.users.push(User { username, password, playlists: Vec::new(), main_urls: Vec::new() });
	let json = serde_json::to_string_pretty(&user_list).map_err(|e| e.to_string())?;
	std::fs::write(&path, json).map_err(|e| e.to_string())?;
	Ok(())
}


#[command]
fn autenticar_usuario(username: String, password: String) -> Result<(), String> {
	let path = get_project_root().join("user_data/user.json");
	// Garante que o diretório existe
	if let Some(parent) = path.parent() {
		std::fs::create_dir_all(parent).map_err(|e| format!("Erro ao criar diretório: {}", e))?;
	}
	println!("[DEBUG] Lendo user.json em: {}", path.display());
	println!("[DEBUG] Username recebido para login: '{}', senha: '{}'", username, password);
	let user_list: UserList = if let Ok(data) = std::fs::read_to_string(&path) {
		serde_json::from_str(&data).unwrap_or_default()
	} else {
		UserList::default()
	};
	let usernames: Vec<String> = user_list.users.iter().map(|u| u.username.clone()).collect();
	println!("[DEBUG] Usuários disponíveis no arquivo: {:?}", usernames);
	if let Some(user) = user_list.users.iter().find(|u| u.username == username) {
		if user.password == password {
			println!("[DEBUG] Login bem-sucedido para usuário: {}", username);
			Ok(())
		} else {
			println!("[ERRO] Senha incorreta para o usuário: {}", username);
			Err("Senha incorreta".to_string())
		}
	} else {
		println!("[ERRO] Usuário não encontrado: {}", username);
		Err("Usuário não encontrado".to_string())
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
	let mut path = get_project_root();
	path.push("Vídeos baixados");
		// Valida o Content-Type
		if let Some(content_type) = response.headers().get("content-type") {
			let ct = content_type.to_str().unwrap_or("");
			if !ct.starts_with("video/") && !ct.contains("octet-stream") {
				println!("[ERRO] Content-Type inesperado: {}", ct);
				return Err(format!("O link não retorna um vídeo válido. Content-Type: {}", ct));
			}
		}
	println!("[DEBUG] Pasta destino: {:?}", path);
	if let Err(e) = std::fs::create_dir_all(&path) {
		println!("[ERRO] Falha ao criar pasta: {:?} => {}", path, e);
		return Err(format!("Erro ao criar pasta: {}", e));
	}
	path.push(ext);
	println!("[DEBUG] Caminho final do arquivo: {:?}", path);
	let mut file = match File::create(&path) {
		Ok(f) => f,
		Err(e) => {
			println!("[ERRO] Falha ao criar arquivo: {:?} => {}", path, e);
			return Err(format!("Erro ao criar arquivo: {}", e));
		}
	};
	let bytes = match response.bytes() {
		Ok(b) => b,
		Err(e) => {
			println!("[ERRO] Falha ao ler bytes do download: {}", e);
			return Err(format!("Erro ao ler bytes: {}", e));
		}
	};
	let downloaded = bytes.len() as u64;
	if let Err(e) = file.write_all(&bytes) {
		println!("[ERRO] Falha ao salvar arquivo: {:?} => {}", path, e);
		return Err(format!("Erro ao salvar: {}", e));
	}
	// Verifica se o arquivo foi realmente criado e tem tamanho > 0
	match std::fs::metadata(&path) {
		Ok(meta) => {
			if meta.len() == 0 {
				println!("[ERRO] Arquivo criado mas está vazio: {:?}", path);
				return Err("Arquivo criado mas está vazio".to_string());
			}
		}
		Err(e) => {
			println!("[ERRO] Não foi possível obter metadata do arquivo: {:?} => {}", path, e);
			return Err(format!("Arquivo não criado corretamente: {}", e));
		}
	}
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
	let mut path = get_project_root();
	path.push("Vídeos baixados");
	println!("[DEBUG] Pasta destino HLS: {:?}", path);
	if let Err(e) = std::fs::create_dir_all(&path) {
		println!("[ERRO] Falha ao criar pasta HLS: {:?} => {}", path, e);
		return Err(format!("Erro ao criar pasta: {}", e));
	}
	path.push(filename);
	println!("[DEBUG] Caminho final do arquivo HLS: {:?}", path);
	if ts_urls.is_empty() {
		return Err("Nenhum segmento .ts válido encontrado na playlist".to_string());
	}
	let mut file = match File::create(&path) {
		Ok(f) => f,
		Err(e) => {
			println!("[ERRO] Falha ao criar arquivo HLS: {:?} => {}", path, e);
			return Err(format!("Erro ao criar arquivo: {}", e));
		}
	};
	for (i, ts_url) in ts_urls.iter().enumerate() {
		println!("[HLS] Baixando segmento {}/{}", i+1, ts_urls.len());
		let seg = client.get(ts_url)
			.send().map_err(|e| format!("Erro ao baixar segmento: {}", e))?
			.bytes().map_err(|e| format!("Erro ao ler segmento: {}", e))?;
		file.write_all(&seg).map_err(|e| format!("Erro ao salvar segmento: {}", e))?;
		// Emitir progresso para o frontend e terminal
		let progress = ((i + 1) as f64 / ts_urls.len() as f64 * 100.0).round() as u8;
		println!("[HLS] Progresso: {}/{} ({}%)", i+1, ts_urls.len(), progress);
		let _ = window.emit("download_progress", match id {
			Some(id) => serde_json::json!({ "id": id, "progress": progress }),
			None => serde_json::json!({ "filename": filename, "progress": progress })
		});
	}
	// Verifica se o arquivo foi realmente criado e tem tamanho > 0
	match std::fs::metadata(&path) {
		Ok(meta) => {
			if meta.len() == 0 {
				println!("[ERRO] Arquivo HLS criado mas está vazio: {:?}", path);
				return Err("Arquivo HLS criado mas está vazio".to_string());
			}
		}
		Err(e) => {
			println!("[ERRO] Não foi possível obter metadata do arquivo HLS: {:?} => {}", path, e);
			return Err(format!("Arquivo HLS não criado corretamente: {}", e));
		}
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
	// Sempre salva o HTML baixado para análise
	let mut path = get_project_root();
	path.push("user_data");
	std::fs::create_dir_all(&path).ok();
	path.push("last_player_html.html");
	if let Err(e) = std::fs::write(&path, &html) {
		println!("[ERRO] Falha ao salvar HTML do player para análise: {}", e);
	} else {
		println!("[INFO] HTML do player salvo em: {}", path.display());
	}
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
		None => return Err("Não foi possível extrair o link .m3u8 do player. O HTML foi salvo em user_data/last_player_html.html para análise.".to_string()),
	};
	println!("[EXTRAÇÃO] Link .m3u8 extraído: {}", m3u8_url);
	baixar_hls_emit(window, m3u8_url, output, id)
}

#[command]
fn baixar_video_tauri(window: Window, url: String, filename: String, id: Option<u64>) -> Result<(), String> {
	if url.contains("player.jmvstream.com") {
		// Sempre tenta extrair o .m3u8 do player
		baixar_player_jmvstream(&window, &url, &filename, id)
	} else if url.ends_with(".m3u8") || url.contains(".m3u8?") {
		baixar_hls_emit(&window, &url, &filename, id)
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
			   .invoke_handler(tauri::generate_handler![
				   baixar_video_tauri,
				   listar_videos_baixados,
				   cadastrar_usuario,
				   autenticar_usuario,
				   baixar_em_cascata,
				   get_main_urls,
				   add_main_url,
				   open_download_folder,
				   update_main_url_title
			   ])
		       .run(tauri::generate_context!())
		       .expect("error while running tauri application");
}

