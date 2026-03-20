use tauri::{LogicalSize, WebviewWindow};
use tokio::time::sleep;

/// Ajusta a janela para metade da largura do monitor e altura útil (descontando barra de tarefas),
/// compensando bordas do Windows.
pub async fn ajustar_janela_metade_tela(window: WebviewWindow, barra_tarefas: Option<f64>) {
    let screens = window.available_monitors().unwrap();
    for (_i, _m) in screens.iter().enumerate() {
    }
    // Seleciona o monitor de maior largura
    let (_monitor_idx, _monitor, screen_width, screen_height, _scale_factor, pos_x, pos_y) = screens.iter().enumerate()
        .map(|(_i, _m)| (_i, _m, _m.size().width as f64, _m.size().height as f64, _m.scale_factor(), _m.position().x, _m.position().y))
        .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
        .unwrap();
    let half_width = (screen_width / 2.0).round();
    let taskbar_height = barra_tarefas.unwrap_or(52.0); // valor padrão
    let usable_height = screen_height - taskbar_height;
    // Ajusta a janela para o monitor correto
    let incremento_altura = 12.0;
    let alvo_altura = usable_height + incremento_altura;
    window.set_size(tauri::Size::Logical(LogicalSize {
        width: half_width,
        height: alvo_altura,
    })).unwrap();
    // Deslocar 7px para a esquerda (5 + 2) e colar no topo
    let desloc_x = pos_x - 7;
    let desloc_y = pos_y;
    window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
        x: desloc_x,
        y: desloc_y,
    })).unwrap();

    // Ajuste fino após delay para compensar bordas
    let window2 = window.clone();
    tauri::async_runtime::spawn(async move {
        sleep(std::time::Duration::from_millis(400)).await;
        if let Ok(win_size) = window2.outer_size() {
            let largura_atual = win_size.width as f64;
            let altura_atual = win_size.height as f64;
            let diff_largura = half_width - largura_atual;
            let diff_altura = alvo_altura - altura_atual;
            let largura_corrigida = half_width + diff_largura;
            let altura_corrigida = alvo_altura + diff_altura;
            let _ = window2.set_size(tauri::Size::Logical(LogicalSize {
                width: largura_corrigida,
                height: altura_corrigida,
            }));
            let desloc_x = pos_x - 7;
            let desloc_y = pos_y;
            let _ = window2.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: desloc_x,
                y: desloc_y,
            }));
            if let Ok(_win_size2) = window2.outer_size() {
            }
        }
    });
}
