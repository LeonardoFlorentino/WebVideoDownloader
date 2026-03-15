
use tauri::command;
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

fn baixar_hls(m3u8_url: &str, filename: &str) -> Result<(), String> {
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
		return baixar_hls(url, filename);
	}
	if ts_urls.is_empty() {
		return Err("Nenhum segmento .ts encontrado na playlist".to_string());
	}
	let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
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
	}
	println!("[HLS] Download finalizado: {:?}", path);
	Ok(())
}

fn baixar_player_jmvstream(player_url: &str, output: &str) -> Result<(), String> {
	println!("[EXTRAÇÃO] Baixando HTML do player: {}", player_url);
	let client = reqwest::blocking::Client::builder()
		.user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
		.build()
		.map_err(|e| format!("Erro ao criar client: {}", e))?;
	let html = client.get(player_url)
		.header("Referer", player_url)
		.send().map_err(|e| format!("Erro ao baixar HTML do player: {}", e))?
		.text().map_err(|e| format!("Erro ao ler HTML do player: {}", e))?;
	let re = Regex::new(r#"src":"([^"]+\.m3u8[^"]*)""#).unwrap();
	let m3u8_url = if let Some(cap) = re.captures(&html) {
		cap.get(1).map(|m| m.as_str())
	} else {
		None
	};
	let m3u8_url = match m3u8_url {
		Some(url) => url,
		None => return Err("Não foi possível extrair o link .m3u8 do player".to_string()),
	};
	println!("[EXTRAÇÃO] Link .m3u8 extraído: {}", m3u8_url);
	baixar_hls(m3u8_url, output)
}

#[command]
fn baixar_video_tauri(url: String, filename: String) -> Result<(), String> {
	if url.ends_with(".m3u8") || url.contains(".m3u8?") {
		baixar_hls(&url, &filename)
	} else if url.contains("player.jmvstream.com") {
		baixar_player_jmvstream(&url, &filename)
	} else {
		baixar_video(&url, &filename)
	}
}

fn main() {
	tauri::Builder::default()
		.invoke_handler(tauri::generate_handler![baixar_video_tauri])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}

