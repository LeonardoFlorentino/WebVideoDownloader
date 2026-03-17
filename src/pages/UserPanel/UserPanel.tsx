



import React from 'react';
import { ToastContainer, toast } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';
import { useNavigate } from 'react-router-dom';
import { Container, Card, Header, IconCircle, Username, Button } from './UserPanel.styles';
import { TopBar, NavButton } from '../Home/Home.styles';
import type { Playlist } from '@/types/playlist';

interface UserPanelProps {
  username: string;
}

const UserPanel: React.FC<UserPanelProps> = ({ username }) => {
  const navigate = useNavigate();
  // Persistência por usuário (username) no localStorage
  const [playlists, setPlaylists] = React.useState<Playlist[]>(() => {
    try {
      const data = localStorage.getItem(`playlists_${username}`);
      return data ? JSON.parse(data) : [];
    } catch {
      return [];
    }
  });
  const [showModal, setShowModal] = React.useState<boolean>(false);
  const [modalTitle, setModalTitle] = React.useState<string>('');
  const [modalLinks, setModalLinks] = React.useState<string[]>(['']);
  const [editId, setEditId] = React.useState<string | null>(null);
  // Removido showToast, usaremos react-toastify

  const handleLogout = (): void => {
    navigate('/login');
  };
  const handleGoHome = (): void => {
    navigate('/');
  };

  // Abrir modal para criar nova playlist
  const openCreateModal = (): void => {
    setModalTitle('');
    setModalLinks(['']);
    setEditId(null);
    setShowModal(true);
  };

  // Abrir modal para editar playlist existente
  const openEditModal = (playlist: Playlist): void => {
    setModalTitle(playlist.name);
    setModalLinks(playlist.links.map(l => l.url).length ? playlist.links.map(l => l.url) : ['']);
    setEditId(playlist.id);
    setShowModal(true);
  };

  // Adiciona/remover campos de link na modal
  const handleAddLinkField = (): void => setModalLinks(links => [...links, '']);
  const handleRemoveLinkField = (idx: number): void => setModalLinks(links => links.length > 1 ? links.filter((_, i) => i !== idx) : links);
  const handleChangeLinkField = (idx: number, value: string): void => setModalLinks(links => links.map((l, i) => i === idx ? value : l));

  // Salvar playlist (criar ou editar)
  const handleSavePlaylist = (e: React.FormEvent<HTMLFormElement>): void => {
    e.preventDefault();
    const title = modalTitle.trim();
    const urls = modalLinks.map(u => u.trim()).filter(Boolean);
    if (!title) return;
    let newPlaylists: Playlist[];
    if (editId) {
      newPlaylists = playlists.map(pl => pl.id === editId ? { ...pl, name: title, links: urls.map(url => ({ id: Date.now().toString() + Math.random(), url })) } : pl);
    } else {
      newPlaylists = [
        { id: Date.now().toString(), name: title, links: urls.map(url => ({ id: Date.now().toString() + Math.random(), url })) },
        ...playlists
      ];
    }
    setPlaylists(newPlaylists);
    localStorage.setItem(`playlists_${username}`, JSON.stringify(newPlaylists));
    setShowModal(false);
    toast.success('Playlist salva com sucesso!', {
      position: 'top-center',
      autoClose: 2000,
      hideProgressBar: true,
      closeOnClick: true,
      pauseOnHover: false,
      draggable: false,
      theme: 'dark',
    });
  };
  // Atualizar localStorage sempre que playlists mudar (caso haja outras edições futuras)
  React.useEffect(() => {
    localStorage.setItem(`playlists_${username}`, JSON.stringify(playlists));
  }, [playlists, username]);

  return (
    <Container style={{ minHeight: '100vh', width: '100vw', padding: 0, margin: 0, display: 'flex', flexDirection: 'column' }}>
      {/* TopBar fixo no topo, ocupando 100% da largura */}
      <TopBar style={{
        position: 'fixed',
        top: 0,
        left: 0,
        width: '100vw',
        zIndex: 100,
        justifyContent: 'space-between',
        background: 'transparent',
        borderBottom: 'none',
        boxShadow: 'none',
        paddingLeft: 32,
        paddingRight: 32
      }}>
        <NavButton onClick={handleGoHome}>Home</NavButton>
        <NavButton onClick={handleLogout}>Logout</NavButton>
      </TopBar>
      {/* Card de boas-vindas */}
      <div style={{ width: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', marginTop: 96 }}>
        <Card>
          <Header>
            <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
              <IconCircle>
                <svg xmlns='http://www.w3.org/2000/svg' className='h-6 w-6' fill='none' viewBox='0 0 24 24' stroke='white'><path strokeLinecap='round' strokeLinejoin='round' strokeWidth={2} d='M16 21v-2a4 4 0 00-8 0v2M12 11a4 4 0 100-8 4 4 0 000 8z' /></svg>
              </IconCircle>
              <Username>Bem-vindo, {username}</Username>
            </div>
          </Header>
        </Card>
      </div>
      {/* Botão de criar playlist logo abaixo do card de boas-vindas */}
      <div style={{ width: '100%', display: 'flex', justifyContent: 'center', margin: '32px 0 0 0' }}>
        <Button onClick={openCreateModal} style={{ fontSize: 18, padding: '10px 32px' }}>Criar Playlist</Button>
      </div>
      {/* Cartões de playlists */}
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 32, justifyContent: 'center', margin: '32px 0' }}>
        {playlists.length === 0 && <div style={{ color: '#aaa', fontSize: 18 }}>Nenhuma playlist criada ainda.</div>}
        {playlists.map(pl => (
          <div key={pl.id} style={{ background: '#23234a', borderRadius: 16, boxShadow: '0 2px 12px #0002', padding: 24, minWidth: 320, maxWidth: 360, display: 'flex', flexDirection: 'column', gap: 12 }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 8 }}>
              <div style={{ fontWeight: 700, fontSize: 20, color: '#fff' }}>{pl.name}</div>
              <Button onClick={() => openEditModal(pl)} style={{ padding: '6px 18px', fontSize: 15 }}>Editar</Button>
            </div>
            <div style={{ color: '#b3b3ff', fontSize: 15, marginTop: 6, marginBottom: 8 }}>Links:</div>
            <ul style={{ listStyle: 'none', padding: 0, margin: 0 }}>
              {pl.links.length === 0 && <li style={{ color: '#aaa' }}>Nenhum link</li>}
              {pl.links.map(link => (
                <li key={link.id} style={{ background: '#363759', color: '#fff', borderRadius: 8, padding: '6px 10px', marginBottom: 6, wordBreak: 'break-all' }}>{link.url}</li>
              ))}
            </ul>
          </div>
        ))}
      </div>
      {/* Toast de sucesso */}
      <ToastContainer />
      {/* Modal de criar/editar playlist */}
      {showModal && (
        <div style={{
          position: 'fixed', top: 0, left: 0, width: '100vw', height: '100vh', background: '#000a', zIndex: 1000,
          display: 'flex', alignItems: 'center', justifyContent: 'center'
        }}>
          <form onSubmit={handleSavePlaylist} style={{ background: '#23234a', borderRadius: 16, padding: 32, minWidth: 340, maxWidth: 420, boxShadow: '0 4px 32px #0008', display: 'flex', flexDirection: 'column', gap: 18 }}>
            <div style={{ fontWeight: 700, fontSize: 22, color: '#fff', marginBottom: 8 }}>{editId ? 'Editar Playlist' : 'Nova Playlist'}</div>
            <input
              type="text"
              placeholder="Título da playlist"
              value={modalTitle}
              onChange={e => setModalTitle(e.target.value)}
              style={{
                padding: '10px 14px', borderRadius: 8, border: 'none', outline: 'none', background: '#363759', color: '#fff', fontSize: 17, marginBottom: 6
              }}
              required
            />
            <div style={{ color: '#b3b3ff', fontWeight: 600, fontSize: 15, marginBottom: 4 }}>URLs dos vídeos</div>
            {modalLinks.map((url, idx) => (
              <div key={idx} style={{ display: 'flex', gap: 8, marginBottom: 4 }}>
                <input
                  type="text"
                  placeholder={`URL #${idx + 1}`}
                  value={url}
                  onChange={e => handleChangeLinkField(idx, e.target.value)}
                  style={{
                    flex: 1, padding: '8px 12px', borderRadius: 8, border: 'none', outline: 'none', background: '#363759', color: '#fff', fontSize: 16
                  }}
                  required={idx === 0}
                />
                {modalLinks.length > 1 && (
                  <button type="button" onClick={() => handleRemoveLinkField(idx)} style={{ background: '#ff5e5e', color: '#fff', border: 'none', borderRadius: 8, padding: '0 10px', cursor: 'pointer', fontWeight: 700, fontSize: 18 }}>-</button>
                )}
                {idx === modalLinks.length - 1 && (
                  <button type="button" onClick={handleAddLinkField} style={{ background: '#6c63ff', color: '#fff', border: 'none', borderRadius: 8, padding: '0 12px', cursor: 'pointer', fontWeight: 700, fontSize: 18 }}>+</button>
                )}
              </div>
            ))}
            <div style={{ display: 'flex', gap: 12, marginTop: 10 }}>
              <Button type="submit" style={{ flex: 1, fontSize: 17 }}>{editId ? 'Salvar' : 'Criar'}</Button>
              <Button type="button" onClick={() => setShowModal(false)} style={{ background: '#363759', color: '#fff', fontSize: 17 }}>Cancelar</Button>
            </div>
          </form>
        </div>
      )}
    </Container>
  );
}

export default UserPanel;
