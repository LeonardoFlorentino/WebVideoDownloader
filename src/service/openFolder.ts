import { invoke } from "@tauri-apps/api/core";

export async function openDownloadFolder(playlist: string) {
  return invoke("open_download_folder", { playlist });
}
