use reqwest::blocking::Client;

#[allow(dead_code)]
pub fn get_title_from_url(url: &str) -> Result<String, String> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0")
        .build()
        .map_err(|e| format!("Erro ao criar client: {}", e))?;

    let html = client
        .get(url)
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("Erro ao baixar pagina: {}", e))?
        .text()
        .map_err(|e| format!("Erro ao ler HTML: {}", e))?;

    let lower = html.to_lowercase();
    let start = lower.find("<title>").ok_or("Titulo nao encontrado")? + 7;
    let end = lower[start..]
        .find("</title>")
        .map(|idx| start + idx)
        .ok_or("Titulo nao encontrado")?;

    let title = html[start..end].trim();
    if title.is_empty() {
        Err("Titulo vazio".to_string())
    } else {
        Ok(title.to_string())
    }
}