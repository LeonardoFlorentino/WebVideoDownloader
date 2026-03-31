import { invoke } from "@tauri-apps/api/core";

// Chama o comando correto do backend para pausar e atualizar status
export async function pausarDownloadTauri(username: string, url: string, savePath: string) {
  return invoke("pausar_download", { username, url, savePath });
}
