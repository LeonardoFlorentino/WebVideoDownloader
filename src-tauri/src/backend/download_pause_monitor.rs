use std::sync::{Arc, atomic::AtomicBool};
use std::thread;

/// Inicia uma thread para monitorar o status de pausa de um download.
/// Quando o status do progresso for alterado para "pausado", o AtomicBool será setado para true.
pub fn start_pause_monitor(url_string: String, should_stop: Arc<AtomicBool>) {
    thread::spawn(move || {
        use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
        static MONITOR_COUNTER: once_cell::sync::Lazy<AtomicUsize> = once_cell::sync::Lazy::new(|| AtomicUsize::new(0));
        let mut last_status = String::new();
        loop {
            std::thread::sleep(std::time::Duration::from_millis(300));
            let count = MONITOR_COUNTER.fetch_add(1, AtomicOrdering::SeqCst);
            if let Some(prog) = crate::backend::download_progress::get_progress(&url_string) {
                if prog.status != last_status {
                    println!("[PAUSE MONITOR] status for {}: {}", url_string, prog.status);
                    last_status = prog.status.clone();
                } else if count % 10 == 0 {
                    println!("[PAUSE MONITOR] status for {}: {} (periodic)", url_string, prog.status);
                }
                if prog.status == "pausado" {
                    println!("[PAUSE MONITOR] Detected pause for {}!", url_string);
                    should_stop.store(true, std::sync::atomic::Ordering::SeqCst);
                    break;
                }
            } else if count % 10 == 0 {
                println!("[PAUSE MONITOR] No progress found for {} (periodic)", url_string);
            }
        }
    });
}
