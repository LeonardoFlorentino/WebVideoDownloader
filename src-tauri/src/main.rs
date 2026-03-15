// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;

fn main() {
  let args: Vec<String> = env::args().collect();
  if args.len() > 1 {
    // Espera: cargo run -- <URL> [nome_arquivo]
    let url = args[1].clone();
    let filename = if args.len() > 2 { Some(args[2].clone()) } else { None };
    println!("[CLI] Iniciando download: {}", url);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(app_lib::baixar_video_cli(url, filename));
    match result {
      Ok(path) => println!("[CLI] Download concluído: {}", path),
      Err(e) => println!("[CLI] Erro: {}", e),
    }
    return;
  }
  app_lib::run();
}
