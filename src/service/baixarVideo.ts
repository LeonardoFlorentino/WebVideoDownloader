import { invoke } from "@tauri-apps/api/core";

export async function baixarVideoTauri(
  id: string,
  url: string,
  savePath: string,
) {
  return invoke("start_download", {
    id,
    url,
    savePath,
  });
}
