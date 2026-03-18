export type MainUrl = { url: string; filename: string };

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
  return invoke("get_main_urls", { username });
}
export async function addMainUrl(
  username: string,
  url: string,
  filename?: string,
) {
  return invoke("add_main_url", { username, url, filename });
}
import { invoke } from "@tauri-apps/api/core";
import type { Download } from "@/types/download";

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

export async function baixarEmCascata(playlist: string, urls: string[]) {
  return invoke("baixar_em_cascata", { playlist, urls });
}
