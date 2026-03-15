
import { useState, useRef, useEffect } from 'react';
import { FaPlus, FaTrash } from 'react-icons/fa';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { Download } from './types/download';
import type { VideoInfo } from './types/video';
import DownloadList from './components/DownloadList';
import VideoList from './components/VideoList';
import { listDownloadedVideos } from './lib/listVideos';

import { FaVideo, FaDownload } from 'react-icons/fa';
import './App.css';

function getExtensionFromUrl(url: string): string {
  try {
    const u = new URL(url);
    const pathname = u.pathname;
    const ext = pathname.split('.').pop();
    if (ext && ext.length <= 5) return ext;
    return 'mp4';
  } catch {
    return 'mp4';
  }
}





function App() {
  const [urls, setUrls] = useState<string[]>([""]);
  const [downloads, setDownloads] = useState<Download[]>([]);
  const [videos, setVideos] = useState<VideoInfo[]>([]);
  const downloadRefs = useRef<Record<number, { canceled: boolean }>>({});

  const handleDownload = async (): Promise<void> => {
    for (const url of urls) {
      if (!url.trim()) continue;
      const ext = getExtensionFromUrl(url);
      const id = Date.now();
      const filename = `video_${id}.${ext}`;
      const newDownload: Download = {
        id,
        url,
        ext,
        filename,
        progress: 0,
        status: 'Baixando',
        canceled: false
      };
      setDownloads((ds: Download[]) => [...ds, newDownload]);
      downloadRefs.current[id] = { canceled: false };
      try {
        setDownloads((ds: Download[]) => ds.map((d: Download) => d.id === id ? { ...d, status: 'Baixando', progress: 0 } : d));
        await invoke('baixar_video_tauri', { url, filename, id });
        setDownloads((ds: Download[]) => ds.map((d: Download) => d.id === id ? { ...d, progress: 100, status: 'Concluído' } : d));
      } catch (err: unknown) {
        let msg = 'Erro';
        if (typeof err === 'object' && err !== null && 'toString' in err && typeof (err as { toString: unknown }).toString === 'function') {
          msg = (err as { toString: () => string }).toString();
        } else if (typeof err === 'string') {
          msg = err;
        }
        setDownloads((ds: Download[]) => ds.map((d: Download) => d.id === id ? { ...d, status: `Erro: ${msg}`, progress: 0 } : d));
      }
    }
    setUrls([""]);
  };

  // Escuta eventos de progresso do backend Tauri
  useEffect(() => {
    const unlistenPromise = listen('download_progress', (event: { payload: { id: number; progress: number } }) => {
      const { id, progress } = event.payload;
      setDownloads((ds: Download[]) => ds.map((d: Download) => d.id === id ? { ...d, progress, status: progress === 100 ? 'Concluído' : d.status } : d));
    });
    return () => {
      unlistenPromise.then((unlisten: () => void) => unlisten());
    };
  }, []);

  const cancelDownload = (id: number) => {
    downloadRefs.current[id].canceled = true;
    setDownloads((ds: Download[]) => ds.map((d: Download) => d.id === id ? { ...d, status: 'Cancelado' } : d));
  };


  // Buscar vídeos baixados ao iniciar
  useEffect(() => {
    listDownloadedVideos().then(setVideos);
  }, []);

  const handleOpenFolder = (video: VideoInfo) => {
    invoke('shell_open', { path: video.path });
  };

  const handleShowDetails = (video: VideoInfo) => {
    alert(`Detalhes do vídeo:\nNome: ${video.name}\nCaminho: ${video.path}`);
  };

  return (
    <div className="app-main">
      <header className="app-header" style={{ display: 'flex', alignItems: 'center', gap: 16, padding: '24px 0' }}>
        <span style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
          <FaVideo size={48} color="#6c63ff" style={{ filter: 'drop-shadow(0 2px 6px #b3b3ff)' }} />
          <FaDownload size={40} color="#43b581" style={{ filter: 'drop-shadow(0 2px 6px #b3ffb3)' }} />
        </span>
        <h1 style={{ margin: 0, fontWeight: 700, fontSize: 32, letterSpacing: 1 }}>Convergencia Downloader</h1>
      </header>
      <section className="download-section">
        <div className="download-input-group" style={{ display: 'flex', flexDirection: 'column', gap: 12, alignItems: 'center', marginBottom: 24 }}>
          <div style={{ width: '100%', maxWidth: 520, background: '#23243a', borderRadius: 12, padding: 20, boxShadow: '0 2px 12px #0002' }}>
            <label style={{ fontWeight: 600, fontSize: 18, color: '#b3b3ff', marginBottom: 8, display: 'block' }}>Cole uma ou mais URLs de vídeo:</label>
            {urls.map((u, i) => (
              <div key={i} style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
                <input
                  type="text"
                  placeholder={`URL do vídeo #${i + 1}`}
                  value={u}
                  onChange={e => setUrls(urls => urls.map((val, idx) => idx === i ? e.target.value : val))}
                  style={{ flex: 1, padding: 10, borderRadius: 6, border: '1px solid #444', background: '#18192a', color: '#fff', fontSize: 16 }}
                />
                {urls.length > 1 && (
                  <button onClick={() => setUrls(urls => urls.filter((_, idx) => idx !== i))} style={{ background: 'none', border: 'none', color: '#ff5e5e', cursor: 'pointer' }} title="Remover campo">
                    <FaTrash />
                  </button>
                )}
                {i === urls.length - 1 && (
                  <button onClick={() => setUrls(urls => [...urls, ""])} style={{ background: 'none', border: 'none', color: '#6c63ff', cursor: 'pointer' }} title="Adicionar campo">
                    <FaPlus />
                  </button>
                )}
              </div>
            ))}
            <button
              className="download-btn"
              onClick={handleDownload}
              disabled={urls.every(u => !u.trim())}
              style={{ marginTop: 12, width: '100%', padding: 12, fontSize: 18, fontWeight: 700, borderRadius: 8, background: 'linear-gradient(90deg, #6c63ff 60%, #43b581 100%)', color: '#fff', border: 'none', boxShadow: '0 2px 8px #0002', cursor: 'pointer' }}
            >
              Baixar vídeo(s)
            </button>
          </div>
        </div>
        <DownloadList downloads={downloads} onCancel={cancelDownload} />
      </section>
      <section className="videos-section">
        <h2>Vídeos baixados</h2>
        <VideoList
          videos={videos}
          onOpenFolder={handleOpenFolder}
          onShowDetails={handleShowDetails}
        />
      </section>
    </div>
  );
}

export default App
