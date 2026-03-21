import { invoke } from "@tauri-apps/api/core";

export async function pausarDownloadTauri(id: string) {
  return invoke("pause_download", { id });
}
