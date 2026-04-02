import { useState } from 'react';
import {
  Panel,
  PlaylistCard,
  PlaylistHeader,
  Info,
  Title,
  Status,
  Button,
  DeleteButton,
  ButtonGroup,
  Empty,
  ModalOverlay,
  ModalContent,
  ModalTitle,
  ModalText,
  ModalActions,
  ConfirmButton,
  CancelButton,
} from './PlaylistPanel.styles';

interface Playlist {
  id: string;
  name: string;
  downloaded?: boolean;
}

interface PlaylistPanelProps {
  playlists: Playlist[];
  onDownload: (id: string, name: string) => void;
  onDelete: (id: string) => void;
}

export default function PlaylistPanel({
  playlists,
  onDownload,
  onDelete,
}: PlaylistPanelProps) {
  const [deleteConfirm, setDeleteConfirm] = useState<string | null>(null);

  const handleDeleteClick = (id: string) => {
    setDeleteConfirm(id);
  };

  const handleConfirmDelete = async () => {
    if (deleteConfirm) {
      await onDelete(deleteConfirm);
      setDeleteConfirm(null);
    }
  };

  return (
    <>
      <Panel>
        {playlists.length === 0 ? (
          <Empty>Nenhuma playlist cadastrada.</Empty>
        ) : (
          playlists.map((pl) => (
            <PlaylistCard key={pl.id}>
              <PlaylistHeader>
                <Info>
                  <Title>{pl.name}</Title>
                  <Status downloaded={pl.downloaded || false}>
                    {pl.downloaded ? 'Baixada' : 'Pendente'}
                  </Status>
                </Info>
                <ButtonGroup>
                  <Button
                    disabled={pl.downloaded}
                    onClick={() => onDownload(pl.id, pl.name)}
                  >
                    Baixar
                  </Button>
                  <DeleteButton onClick={() => handleDeleteClick(pl.id)}>
                    Deletar
                  </DeleteButton>
                </ButtonGroup>
              </PlaylistHeader>
            </PlaylistCard>
          ))
        )}
      </Panel>

      {deleteConfirm && (
        <ModalOverlay onClick={() => setDeleteConfirm(null)}>
          <ModalContent onClick={(e) => e.stopPropagation()}>
            <ModalTitle>Confirmar exclusão</ModalTitle>
            <ModalText>
              Tem certeza de que deseja deletar esta playlist? Esta ação não pode ser desfeita.
            </ModalText>
            <ModalActions>
              <CancelButton onClick={() => setDeleteConfirm(null)}>
                Cancelar
              </CancelButton>
              <ConfirmButton onClick={handleConfirmDelete}>
                Deletar
              </ConfirmButton>
            </ModalActions>
          </ModalContent>
        </ModalOverlay>
      )}
    </>
  );
}
