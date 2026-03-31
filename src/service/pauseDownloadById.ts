import { invoke } from "@tauri-apps/api/core";

// Pausa o download abortando a task pelo id
export async function pauseDownloadById(id: number, url: string) {
  return invoke("integrated_pause_download", { id: String(id), url });
}
