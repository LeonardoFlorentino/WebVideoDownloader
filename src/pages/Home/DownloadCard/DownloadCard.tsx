import React from "react";
import { FiTrash2, FiEdit2 } from "react-icons/fi";
import { EditDownloadModal } from "../../../components/EditDownloadModal";
import {
  CardContainer,
  CardFileInfo,
  CardFileExt,
  CardUrlText,
  CardProgressBar,
  CardProgressTrack,
  CardProgressFill,
  CardStatus,
  CardActions,
  OpenFolderButton,
  CardTopRow,
  CardUrlAndStatus,
} from "./DownloadCard.styles";

interface Download {
  id: number;
  filename: string;
  ext: string;
  url: string;
  status: string;
  total?: number;
  progress: number;
}

interface DownloadCardProps {
  download: Download;
  onOpenFolder: () => void;
  onRemove: (id: number) => void;
  onStartDownload: (id: number) => void;
  editModalOpen: boolean;
  editFilename: string;
  editUrl: string;
  openEditModal: () => void;
  handleEditSave: (filename: string, url: string) => void;
  handleEditCancel: () => void;
}

const DownloadCard: React.FC<DownloadCardProps> = ({
  download,
  onOpenFolder,
  onRemove,
  onStartDownload,
  editModalOpen,
  editFilename,
  editUrl,
  openEditModal,
  handleEditSave,
  handleEditCancel,
}) => {
  // Calcula percentual corretamente a partir de bytes
  const progressPercent =
    download.total && download.total > 0
      ? Math.min((download.progress / download.total) * 100, 100)
      : 0;

  // Normaliza status para tratar "concluido" e "concluído" como equivalentes
  const isConcluido =
    download.status === "concluído" || download.status === "concluido";

  return (
    <CardContainer>
      <CardTopRow>
        <CardFileInfo>
          <span style={{ fontWeight: 600, marginRight: 8 }}>
            {download.filename}
          </span>
          <CardFileExt>{download.ext}</CardFileExt>
        </CardFileInfo>
        <CardUrlAndStatus>
          <CardUrlText>{download.url}</CardUrlText>
          <CardStatus $status={download.status}>
            {isConcluido
              ? "Concluído"
              : download.status === "baixando"
                ? "Baixando..."
                : download.status === "pausado"
                  ? "Pausado"
                  : download.status === "erro"
                    ? "Erro"
                    : "Pendente"}
          </CardStatus>
        </CardUrlAndStatus>
      </CardTopRow>
      <CardProgressBar
        style={{ position: "relative", height: 24, marginTop: 8 }}
      >
        <CardProgressTrack style={{ height: 24 }}>
          <CardProgressFill
            $status={download.status}
            style={{
              width: isConcluido ? "100%" : `${progressPercent}%`,
              height: 24,
              borderRadius: 6,
              position: "absolute",
              left: 0,
              top: 0,
            }}
          />
        </CardProgressTrack>
        <span
          style={{
            position: "absolute",
            left: 0,
            top: 0,
            width: "100%",
            height: 24,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            fontWeight: 700,
            color: "#fff",
            zIndex: 1,
            pointerEvents: "none",
            fontSize: 16,
            letterSpacing: 0.5,
            textShadow: "0 1px 4px #23284d, 0 0 2px #23284d, 0 0 6px #23284d",
          }}
        >
          {isConcluido
            ? "100%"
            : download.status === "pausado"
              ? `${Math.round(progressPercent)}% (Pausado)`
              : `${Math.round(progressPercent)}%`}
        </span>
      </CardProgressBar>
      <CardActions>
                {/* Botão Baixar */}
                {download.status === "pendente" && (
                  <button
                    type="button"
                    title="Baixar vídeo"
                    onClick={() => onStartDownload(download.id)}
                    style={{
                      background: "#6c63ff",
                      color: "#fff",
                      border: "none",
                      borderRadius: 6,
                      padding: "6px 16px",
                      fontWeight: 500,
                      cursor: "pointer",
                      marginRight: 8,
                      fontSize: 16,
                    }}
                  >
                    Baixar
                  </button>
                )}
        <OpenFolderButton
          type="button"
          onClick={onOpenFolder}
          title="Abrir pasta de downloads"
        >
          Abrir Pasta
        </OpenFolderButton>
        <button
          type="button"
          title="Editar nome e URL"
          onClick={openEditModal}
          style={{
            background: "none",
            border: "none",
            color: "#6c63ff",
            borderRadius: 6,
            padding: 6,
            marginRight: 2,
            cursor: "pointer",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            fontSize: 20,
          }}
          aria-label="Editar nome e URL"
        >
          <FiEdit2 size={18} />
        </button>
        <button
          type="button"
          title="Remover da lista"
          onClick={() => onRemove(download.id)}
          disabled={download.status === "baixando"}
          aria-label="Remover da lista"
          style={{
            background: "none",
            border: "none",
            color: "#e53935",
            borderRadius: 6,
            padding: 6,
            cursor: download.status === "baixando" ? "not-allowed" : "pointer",
            opacity: download.status === "baixando" ? 0.6 : 1,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            fontSize: 22,
          }}
        >
          <FiTrash2 size={22} />
        </button>
        <EditDownloadModal
          isOpen={editModalOpen}
          initialFilename={editFilename}
          initialUrl={editUrl}
          onSave={handleEditSave}
          onCancel={handleEditCancel}
        />
      </CardActions>
    </CardContainer>
  );
};

export default DownloadCard;
