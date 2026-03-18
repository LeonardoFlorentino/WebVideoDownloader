import React, { useState } from "react";
import { FiTrash2, FiEdit2 } from "react-icons/fi";
import { EditDownloadModal } from "../../../components/EditDownloadModal/EditDownloadModal";
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

import type { Download } from "@/types/download";

interface DownloadCardProps {
  download: Download;
  onDownload: (id: number) => void;
  onRemove: (id: number) => void;
  onOpenFolder: () => void;
  onEdit?: (id: number, newVals: { filename: string; url: string }) => void;
}

const DownloadCard: React.FC<DownloadCardProps> = ({
  download,
  onDownload,
  onRemove,
  onOpenFolder,
  onEdit,
}) => {
  const [modalOpen, setModalOpen] = useState(false);
  const handleEdit = () => setModalOpen(true);
  const handleModalSave = (filename: string, url: string) => {
    if (onEdit && (filename !== download.filename || url !== download.url)) {
      onEdit(download.id, { filename, url });
    }
    setModalOpen(false);
  };
  const handleModalCancel = () => setModalOpen(false);
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
            {download.status === "concluído"
              ? "Concluído"
              : download.status === "baixando"
                ? "Baixando..."
                : download.status === "erro"
                  ? "Erro"
                  : "Pendente"}
          </CardStatus>
          <button
            type="button"
            title="Editar"
            onClick={handleEdit}
            style={{
              background: "none",
              border: "none",
              color: "#6c63ff",
              marginLeft: 8,
              cursor: "pointer",
              fontSize: 18,
              display: "flex",
              alignItems: "center",
            }}
            disabled={
              download.status === "baixando" || download.status === "concluído"
            }
          >
            <FiEdit2 />
          </button>
        </CardUrlAndStatus>
        <EditDownloadModal
          isOpen={modalOpen}
          initialFilename={download.filename}
          initialUrl={download.url}
          onSave={handleModalSave}
          onCancel={handleModalCancel}
        />
      </CardTopRow>
      <CardProgressBar>
        <CardProgressTrack>
          <CardProgressFill
            $status={download.status}
            style={{ width: `${download.progress}%` }}
          />
        </CardProgressTrack>
      </CardProgressBar>
      <CardActions>
        <OpenFolderButton
          type="button"
          onClick={onOpenFolder}
          title="Abrir pasta de downloads"
        >
          Abrir Pasta
        </OpenFolderButton>
        <button
          type="button"
          onClick={() => onDownload(download.id)}
          disabled={
            download.status === "concluído" || download.status === "baixando"
          }
          style={{
            background: "#6c63ff",
            color: "#fff",
            border: "none",
            borderRadius: 6,
            padding: "6px 16px",
            fontWeight: 500,
            cursor:
              download.status === "concluído" || download.status === "baixando"
                ? "not-allowed"
                : "pointer",
            opacity:
              download.status === "concluído" || download.status === "baixando"
                ? 0.6
                : 1,
            transition: "background 0.2s",
            outline: "none",
          }}
        >
          Baixar
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
      </CardActions>
    </CardContainer>
  );
};

export default DownloadCard;
