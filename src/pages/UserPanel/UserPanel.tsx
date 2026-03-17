
import React from 'react';
import { ToastContainer, toast } from 'react-toastify';
import 'react-toastify/dist/ReactToastify.css';
import { useNavigate } from 'react-router-dom';
import {
  Container, Card, Header, IconCircle, Username,
  TopBarFixed, WelcomeWrapper, CreatePlaylistWrapper, PlaylistsWrapper, PlaylistCard, PlaylistCardTitle, PlaylistLinksLabel, PlaylistLinksList, PlaylistLinkItem, NoPlaylists,
  ModalOverlay, ModalForm, ModalTitle, ModalInput, ModalLinksLabel, ModalLinkFieldWrapper, ModalLinkInput, ModalButton, ModalButtonAdd, ModalActions,
  WelcomeHeaderRow, PlaylistCardHeader, EditButton, CreateButton, SubmitButton, NoLinkItem, CancelButton
} from './UserPanel.styles';
import { NavButton } from '../Home/Home.styles';
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
    <Container>
      <TopBarFixed>
        <NavButton onClick={handleGoHome}>Home</NavButton>
        <NavButton onClick={handleLogout}>Logout</NavButton>
      </TopBarFixed>
      <WelcomeWrapper>
        <Card>
          <Header>
            <WelcomeHeaderRow>
              <IconCircle>
                <svg xmlns='http://www.w3.org/2000/svg' className='h-6 w-6' fill='none' viewBox='0 0 24 24' stroke='white'><path strokeLinecap='round' strokeLinejoin='round' strokeWidth={2} d='M16 21v-2a4 4 0 00-8 0v2M12 11a4 4 0 100-8 4 4 0 000 8z' /></svg>
              </IconCircle>
              <Username>Bem-vindo, {username}</Username>
            </WelcomeHeaderRow>
          </Header>
        </Card>
      </WelcomeWrapper>
      <CreatePlaylistWrapper>
        <CreateButton onClick={openCreateModal}>Criar Playlist</CreateButton>
      </CreatePlaylistWrapper>
      <PlaylistsWrapper>
        {playlists.length === 0 && <NoPlaylists>Nenhuma playlist criada ainda.</NoPlaylists>}
        {playlists.map(pl => (
          <PlaylistCard key={pl.id}>
            <PlaylistCardHeader>
              <PlaylistCardTitle>{pl.name}</PlaylistCardTitle>
              <EditButton onClick={() => openEditModal(pl)}>Editar</EditButton>
            </PlaylistCardHeader>
            <PlaylistLinksLabel>Links:</PlaylistLinksLabel>
            <PlaylistLinksList>
              {pl.links.length === 0 && <NoLinkItem>Nenhum link</NoLinkItem>}
              {pl.links.map(link => (
                <PlaylistLinkItem key={link.id}>{link.url}</PlaylistLinkItem>
              ))}
            </PlaylistLinksList>
          </PlaylistCard>
        ))}
      </PlaylistsWrapper>
      <ToastContainer />
      {showModal && (
        <ModalOverlay>
          <ModalForm onSubmit={handleSavePlaylist}>
            <ModalTitle>{editId ? 'Editar Playlist' : 'Nova Playlist'}</ModalTitle>
            <ModalInput
              type="text"
              placeholder="Título da playlist"
              value={modalTitle}
              onChange={e => setModalTitle(e.target.value)}
              required
            />
            <ModalLinksLabel>URLs dos vídeos</ModalLinksLabel>
            {modalLinks.map((url, idx) => (
              <ModalLinkFieldWrapper key={idx}>
                <ModalLinkInput
                  type="text"
                  placeholder={`URL #${idx + 1}`}
                  value={url}
                  onChange={e => handleChangeLinkField(idx, e.target.value)}
                  required={idx === 0}
                />
                {modalLinks.length > 1 && (
                  <ModalButton type="button" onClick={() => handleRemoveLinkField(idx)}>-</ModalButton>
                )}
                {idx === modalLinks.length - 1 && (
                  <ModalButtonAdd type="button" onClick={handleAddLinkField}>+</ModalButtonAdd>
                )}
              </ModalLinkFieldWrapper>
            ))}
            <ModalActions>
              <SubmitButton type="submit">{editId ? 'Salvar' : 'Criar'}</SubmitButton>
              <CancelButton type="button" onClick={() => setShowModal(false)}>Cancelar</CancelButton>
            </ModalActions>
          </ModalForm>
        </ModalOverlay>
      )}
    </Container>
  );
}

export default UserPanel;
