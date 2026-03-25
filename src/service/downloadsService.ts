import { invoke } from "@tauri-apps/api/core";
import type { Download } from "@/types/download";

export type MainUrl = {
  id: number;
  url: string;
  filename: string;
  status?: string;
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
