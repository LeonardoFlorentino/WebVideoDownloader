use std::collections::HashMap;

pub fn listar_videos_baixados() -> Vec<HashMap<String, String>> {
    let mut result = Vec::new();
    let dir = std::env::current_dir().unwrap_or_default().parent().unwrap_or_else(|| std::path::Path::new("")).join("Vídeos baixados");
    fn walk_dir(dir: &std::path::Path, result: &mut Vec<std::collections::HashMap<String, String>>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    let path_str = path.to_string_lossy().to_string();
                    let mut map = std::collections::HashMap::new();
                    map.insert("name".to_string(), name);
                    map.insert("path".to_string(), path_str);
                    result.push(map);
                } else if path.is_dir() {
                    walk_dir(&path, result);
                }
            }
        }
    }
    walk_dir(&dir, &mut result);
    result
}
