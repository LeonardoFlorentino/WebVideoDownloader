import { invoke } from "@tauri-apps/api/core";
import type { Download } from "@/types/download";

export type MainUrl = {
  id: number;
  url: string;
  filename: string;
  status?: string;
  progress?: number;
};

export async function removeMainUrlById(username: string, id: number) {
  return invoke("remove_main_url_by_id_command", { username, id });
}
export async function getTitleFromUrl(url: string): Promise<string> {
  return invoke("get_title_from_url", { url });
}

export async function updateMainUrlTitle(
  username: string,
  oldUrl: string,
  newUrl: string,
  newFilename: string,
) {
  return invoke("update_main_url_title", {
    username,
    oldUrl,
    newUrl,
    newFilename,
  });
}
export async function getMainUrls(username: string): Promise<MainUrl[]> {
  const result = await invoke("get_main_urls_command", { username });
  // Se vier no formato { ok, data, error }, retorna só data ou []
  if (result && typeof result === "object" && "data" in result) {
    return Array.isArray(result.data) ? result.data : [];
  }
  // Se já for array, retorna direto
  return Array.isArray(result) ? result : [];
}

export async function addMainUrl(
  username: string,
  url: string,
  filename?: string,
) {
  return invoke("add_main_url_command", { username, url, filename });
}

const getDownloadsKey = (username: string) => `downloads_${username}`;

export function loadDownloads(username: string): Download[] {
  try {
    const data = localStorage.getItem(getDownloadsKey(username));
    return data ? JSON.parse(data) : [];
  } catch {
    return [];
  }
}

export function saveDownloads(username: string, downloads: Download[]) {
  localStorage.setItem(getDownloadsKey(username), JSON.stringify(downloads));
}

// Faz polling do progresso de todos os downloads ativos
export async function pollDownloadsProgress(
  downloads: Download[],
): Promise<Partial<Download>[]> {
  // Para cada download, busca progresso pelo backend (Tauri)
  // Aqui, vamos supor que existe um comando Tauri chamado 'get_progress' que recebe a URL
  // e retorna { downloaded, total_size, status }
  // Se não existir, será necessário criar esse comando no backend
  const results = await Promise.all(
    downloads.map(async (d) => {
      try {
        const progress = await invoke<any>("get_progress_command", {
          url: d.url,
        });
        // Unwrap if backend returns { ok, data, error }
        let prog = progress;
        if (
          progress &&
          typeof progress === "object" &&
          "data" in progress &&
          progress.data
        ) {
          prog = progress.data;
        }
        // ÚNICO console.log: mostra o progresso retornado da rota get_progress
        console.log("[PROGRESS get_progress]", { url: d.url, progress: prog });
        // Se vier só { url }, preenche os campos faltantes com zero/default
        if (prog && typeof prog === "object") {
          return {
            id: d.id,
            // Salva como bytes, não percentual!
            progress: typeof prog.downloaded === "number" ? prog.downloaded : 0,
            total: typeof prog.total_size === "number" ? prog.total_size : 0,
            status: typeof prog.status === "string" ? prog.status : "pendente",
            url: typeof prog.url === "string" ? prog.url : d.url,
            filename:
              typeof prog.filename === "string" ? prog.filename : d.filename,
          };
        }
        return {
          id: d.id,
          progress: 0,
          total: 0,
          status: "pendente",
          url: d.url,
          filename: d.filename,
        };
      } catch {
        return { id: d.id };
      }
    }),
  );
  return results;
}
