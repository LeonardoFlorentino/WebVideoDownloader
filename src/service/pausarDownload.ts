import { invoke } from "@tauri-apps/api/core";

// Chama o comando correto do backend para pausar e atualizar status
export async function pausarDownloadTauri(url: string) {
  return invoke("pausar_download", { url });
}
