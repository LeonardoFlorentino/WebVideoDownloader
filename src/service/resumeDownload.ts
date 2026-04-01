import { invoke } from "@tauri-apps/api/core";
import { baixarVideoEspecialTauri } from "./baixarVideo";

export async function resumeDownloadTauri(
  id: string,
  username: string,
  url: string,
  savePath: string,
) {
  try {
    return await invoke("resume_download", { id, username, url, savePath });
  } catch (err: unknown) {
    if (typeof err === "string" && err === "special_video") {
      return baixarVideoEspecialTauri(id, username, url, savePath);
    }
    throw err;
  }
}
