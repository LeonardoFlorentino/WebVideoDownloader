import React, { useState } from 'react';
import {
  Container,
  TopBar,
  NavButton,
  Title,
  UrlForm,
  UrlInput,
  AddButton,
  DownloadList,
  DownloadItem,
  FileInfo,
  FileName,
  FileExt,
  ProgressBar,
  ProgressTrack,
  ProgressFill,
  Status,
  DownloadActions,
  DownloadAllButton,
  EmptyMessage,
  UrlText,
  DownloadButton
} from './Home.styles';

import type { Download } from '@/types/download';


import { useNavigate } from 'react-router-dom';

export default function Home() {
  const [url, setUrl] = useState('');
  const [downloads, setDownloads] = useState<Download[]>([]);
  const [downloadingAll, setDownloadingAll] = useState(false);
  const navigate = useNavigate();

  // Simula adicionar um novo download
  const handleAdd = (e: React.FormEvent) => {
    e.preventDefault();
    if (!url.trim()) return;
    setDownloads(prev => [
      {
        id: Date.now(),
        url,
        filename: `video_${prev.length + 1}`,
        ext: 'mp4',
        progress: 0,
        status: 'pendente',
        canceled: false,
      },
      ...prev
    ]);
    setUrl('');
  };

  const handleDownloadAll = () => {
    setDownloadingAll(true);
    setDownloads(ds => ds.map(d => d.status === 'pendente' ? { ...d, status: 'baixando', progress: 10 } : d));
    setTimeout(() => {
      setDownloads(ds => ds.map(d =>
        d.status === 'baixando'
          ? { ...d, status: 'concluído', progress: 100 }
          : d
      ));
      setDownloadingAll(false);
    }, 2000);
  };

  const handleDownload = (id: number) => {
    setDownloads(ds => ds.map(d => d.id === id ? { ...d, status: 'baixando', progress: 10 } : d));
    setTimeout(() => {
      setDownloads(ds => ds.map(d =>
        d.id === id
          ? { ...d, status: 'concluído', progress: 100 }
          : d
      ));
    }, 2000);
  };

  const handleLogout = () => {
    // Aqui você pode limpar o token/autenticação se necessário
    navigate('/login');
  };

  const handleGoToPanel = () => {
    navigate('/user');
  };

  return (
    <Container>
      <TopBar>
        <div>
          <NavButton onClick={handleGoToPanel}>Painel</NavButton>
        </div>
        <div>
          <NavButton onClick={handleLogout}>Logout</NavButton>
        </div>
      </TopBar>
      <Title>Web Video Downloader</Title>
      <UrlForm onSubmit={handleAdd}>
        <UrlInput
          type="text"
          placeholder="Cole a URL do vídeo aqui..."
          value={url}
          onChange={e => setUrl(e.target.value)}
        />
        <AddButton type="submit" disabled={!url.trim()}>Adicionar</AddButton>
      </UrlForm>
      <DownloadAllButton onClick={handleDownloadAll} disabled={downloads.length === 0 || downloads.every(d => d.status === 'concluído') || downloadingAll}>
        Baixar Todos
      </DownloadAllButton>
      <DownloadList>
        {downloads.length === 0 && <EmptyMessage>Nenhum vídeo adicionado ainda.</EmptyMessage>}
        {downloads.map(d => (
          <DownloadItem key={d.id}>
            <FileInfo>
              <FileName>{d.filename}</FileName>
              <FileExt>{d.ext}</FileExt>
            </FileInfo>
            <div style={{ flex: 2, minWidth: 0 }}>
              <UrlText>{d.url}</UrlText>
              <ProgressBar>
                <ProgressTrack>
                  <ProgressFill style={{ width: `${d.progress}%` }} status={d.status} />
                </ProgressTrack>
              </ProgressBar>
            </div>
            <Status status={d.status}>{d.status === 'concluído' ? 'Concluído' : d.status === 'baixando' ? 'Baixando...' : d.status === 'erro' ? 'Erro' : 'Pendente'}</Status>
            <DownloadActions>
              <DownloadButton
                onClick={() => handleDownload(d.id)}
                disabled={d.status === 'concluído' || d.status === 'baixando'}
              >
                Baixar
              </DownloadButton>
            </DownloadActions>
          </DownloadItem>
        ))}
      </DownloadList>
    </Container>
  );
}
