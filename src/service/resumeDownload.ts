import { invoke } from "@tauri-apps/api/core";
import {
  baixarVideoEspecialTauri,
  type DownloadRequestContext,
} from "./baixarVideo";

export async function resumeDownloadTauri(
  id: string,
  username: string,
  url: string,
  savePath: string,
  context?: DownloadRequestContext,
) {
  try {
    return await invoke("resume_download", {
      id,
      username,
      url,
      savePath,
      progressKey: context?.progressKey,
      source: context?.source,
    });
  } catch (err: unknown) {
    if (typeof err === "string" && err === "special_video") {
      return baixarVideoEspecialTauri(id, username, url, savePath, context);
    }
    throw err;
  }
}
