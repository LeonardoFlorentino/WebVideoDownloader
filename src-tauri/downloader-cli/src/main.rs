use regex::Regex;
mod hls;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// URL do vídeo
    url: String,
    /// Nome do arquivo de saída (opcional)
    #[arg(short, long)]
    output: Option<String>,
}

fn main() {
    let args = Args::parse();
    let url = &args.url;
    let output = args.output.as_deref().unwrap_or("video.mp4");
    let res = if url.ends_with(".m3u8") || url.contains(".m3u8?") {
        hls::baixar_hls(url, output)
    } else if url.contains("player.jmvstream.com") {
        baixar_player_jmvstream(url, output)
    } else {
        baixar_video(url, Some(output))
    };
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
        // Extrai o link .m3u8 do parâmetro src do Popcorn
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
        hls::baixar_hls(m3u8_url, output)
    }
    if let Err(e) = res {
        eprintln!("Erro: {}", e);
        std::process::exit(1);
    }
}

fn baixar_video(url: &str, filename: Option<&str>) -> Result<(), String> {
    println!("[CLI] Iniciando download: {}", url);
    let response = reqwest::blocking::get(url).map_err(|e| format!("Erro ao baixar: {}", e))?;
    if !response.status().is_success() {
        return Err(format!("HTTP {}: {}", response.status().as_u16(), response.status()));
    }
    let total_size = response.content_length().unwrap_or(0);
    let ext = filename.unwrap_or("video.mp4");
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
