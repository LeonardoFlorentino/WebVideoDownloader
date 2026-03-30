import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

// Função para downloads especiais (HLS/JMV)
export async function baixarVideoEspecialTauri(
  id: string,
  username: string,
  url: string,
  savePath: string,
) {
  const window = getCurrentWindow();
  // id pode ser string ou number, backend espera Option<u64>
  const parsedId = isNaN(Number(id)) ? undefined : Number(id);
  return invoke("download_special_video", {
    window,
    username,
    url,
    savePath,
    id: parsedId,
  });
}

// Função principal adaptada: chama start_download, se "special_video" chama baixarVideoEspecialTauri
export async function baixarVideoTauri(
  id: string,
  username: string,
  url: string,
  savePath: string,
) {
  try {
    return await invoke("start_download", {
      id,
      username,
      url,
      savePath,
    });
  } catch (err: any) {
    if (typeof err === "string" && err === "special_video") {
      return baixarVideoEspecialTauri(id, username, url, savePath);
    }
    throw err;
  }
}
