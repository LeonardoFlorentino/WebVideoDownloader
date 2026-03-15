import { invoke } from '@tauri-apps/api/core';
import type { VideoInfo } from '../types/video';

// Função para listar vídeos já baixados na pasta do backend
export async function listDownloadedVideos(): Promise<VideoInfo[]> {
  // O backend deve expor um comando para listar os arquivos da pasta de vídeos baixados
  // Aqui supomos que existe um comando Tauri chamado 'listar_videos_baixados'
  const files: { name: string; path: string }[] = await invoke('listar_videos_baixados');
  // Pode-se adicionar lógica para buscar miniaturas e status
  return files.map(f => ({
    name: f.name,
    path: f.path,
    status: 'Concluído',
    // thumbnail: gerar ou buscar miniatura
  }));
}
