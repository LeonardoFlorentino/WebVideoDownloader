import { invoke } from "@tauri-apps/api/core";

export async function resumeDownloadTauri(
  id: string,
  username: string,
  url: string,
  savePath: string,
) {
  return invoke("resume_download", { id, username, url, savePath });
}
