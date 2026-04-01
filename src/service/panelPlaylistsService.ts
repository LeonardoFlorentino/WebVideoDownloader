import { invoke } from "@tauri-apps/api/core";
import type { Playlist } from "@/types/playlist";

type CommandResult<T> = {
  ok: boolean;
  data?: T;
  error?: string;
};

export async function getPanelPlaylists(username: string): Promise<Playlist[]> {
  const result = (await invoke("get_panel_playlists_command", {
    username,
  })) as CommandResult<Playlist[]>;

  return Array.isArray(result?.data) ? result.data : [];
}

export async function replacePanelPlaylists(
  username: string,
  playlists: Playlist[],
): Promise<void> {
  const result = (await invoke("replace_panel_playlists_command", {
    username,
    playlists,
  })) as CommandResult<null>;

  if (!result?.ok) {
    throw new Error(result?.error || "Erro ao salvar playlists do painel");
  }
}