import { invoke } from "@tauri-apps/api/core";

export async function baixarVideoTauri(
  window: any, // ou remova se não for necessário
  username: string,
  url: string,
  filename: string,
  id?: number,
) {
  return invoke("baixar_video_tauri", {
    window,
    username,
    url,
    filename,
    id,
  });
}
