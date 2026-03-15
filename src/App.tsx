import { useState, useRef, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { Download } from './types/download';
import reactLogo from './assets/react.svg'
import viteLogo from './assets/vite.svg'
import heroImg from './assets/hero.png'
import './App.css'

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
  const [url, setUrl] = useState<string>("");
  const [downloads, setDownloads] = useState<Download[]>([]);
  const downloadRefs = useRef<Record<number, { canceled: boolean }>>({});

  const handleDownload = async (): Promise<void> => {
    if (!url) return;
    const ext = getExtensionFromUrl(url);
    const id = Date.now() + Math.random();
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
    setUrl("");

    try {
      setDownloads((ds: Download[]) => ds.map((d: Download) => d.id === id ? { ...d, status: 'Baixando', progress: 0 } : d));
      await invoke('baixar_video', { url, filename });
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
  };

  // Escuta eventos de progresso do backend Tauri
  useEffect(() => {
    const unlistenPromise = listen('download_progress', (event: { payload: { filename: string; progress: number } }) => {
      const { filename, progress } = event.payload;
      setDownloads((ds: Download[]) => ds.map((d: Download) => d.filename === filename ? { ...d, progress, status: progress === 100 ? 'Concluído' : d.status } : d));
    });
    return () => {
      unlistenPromise.then((unlisten: () => void) => unlisten());
    };
  }, []);

  const cancelDownload = (id: number) => {
    downloadRefs.current[id].canceled = true;
    setDownloads((ds: Download[]) => ds.map((d: Download) => d.id === id ? { ...d, status: 'Cancelado' } : d));
  };

  return (
    <>
      <section id="center">
        <div className="hero">
          <img src={heroImg} className="base" width="170" height="179" alt="" />
          <img src={reactLogo} className="framework" alt="React logo" />
          <img src={viteLogo} className="vite" alt="Vite logo" />
        </div>
        <div style={{ marginTop: 32, marginBottom: 32, textAlign: 'center' }}>
          <h1>Baixar vídeo</h1>
          <input
            type="text"
            placeholder="Cole a URL do vídeo aqui"
            value={url}
            onChange={e => setUrl(e.target.value)}
            style={{ width: 400, padding: 8, fontSize: 16 }}
          />
          <br />
          <button
            style={{ marginTop: 16, padding: '10px 24px', fontSize: 16 }}
            onClick={handleDownload}
            disabled={!url}
          >
            Baixar vídeo
          </button>
        </div>
        {/* Lista de downloads */}
        <div style={{ marginTop: 40, maxWidth: 600, marginLeft: 'auto', marginRight: 'auto' }}>
          {downloads.length > 0 && (
            <h2 style={{ textAlign: 'center', marginBottom: 16 }}>Downloads</h2>
          )}
          {downloads.map(d => (
            <div key={d.id} style={{ background: '#181818', borderRadius: 10, marginBottom: 18, padding: 18, boxShadow: '0 1px 4px #0002', position: 'relative', display: 'flex', alignItems: 'center' }}>
              <div style={{ flex: 1 }}>
                <div style={{ fontWeight: 600, fontSize: 16, color: '#fff' }}>{d.url}</div>
                <div style={{ fontSize: 13, color: '#aaa', marginBottom: 6 }}>Salvar como: <b>.{d.ext}</b></div>
                <div style={{ width: '100%', height: 18, background: '#222', borderRadius: 8, overflow: 'hidden', marginBottom: 4, position: 'relative' }}>
                  <div style={{ width: `${d.progress}%`, height: '100%', background: d.status === 'Cancelado' ? '#b71c1c' : '#4caf50', transition: 'width 0.2s' }} />
                  <span style={{ position: 'absolute', left: '50%', top: 0, transform: 'translateX(-50%)', color: '#fff', fontWeight: 600, fontSize: 12 }}>{d.progress}%</span>
                </div>
                <span style={{ color: d.status === 'Cancelado' ? '#b71c1c' : '#4caf50', fontWeight: 500, fontSize: 13 }}>{d.status}</span>
              </div>
              {d.status === 'Baixando' && (
                <button onClick={() => cancelDownload(d.id)} style={{ marginLeft: 18, background: 'none', border: 'none', color: '#fff', fontSize: 22, cursor: 'pointer', fontWeight: 700, lineHeight: 1 }} title="Cancelar download">×</button>
              )}
            </div>
          ))}
        </div>
      </section>
      <div className="ticks"></div>
    </>
  )
}

export default App
