import { invoke } from "@tauri-apps/api/core";

export async function baixarVideoTauri(
  id: string,
  username: string,
  url: string,
  savePath: string,
) {
  return invoke("start_download", {
    id,
    username,
    url,
    savePath,
  });
}
