import { invoke } from "@tauri-apps/api/core";

export async function pausarDownloadTauri(url: string) {
  return invoke("pausar_download", { url });
}
