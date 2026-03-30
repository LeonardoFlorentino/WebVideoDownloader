import { ClipLoader } from "react-spinners";
import React from "react";
import { FiTrash2, FiPause, FiPlay } from "react-icons/fi";
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
  PauseButton,
  ResumeButton,
} from "./DownloadCard.styles";

import type { Download } from "@/types/download";

interface DownloadCardProps {
  download: Download;
  onDownload: (id: number, action?: "pause" | "resume") => void;
  onRemove: (id: number) => void;
  onOpenFolder: () => void;
  pausando?: boolean;
}

const DownloadCard: React.FC<DownloadCardProps> = ({
  download,
  onDownload,
  onRemove,
  onOpenFolder,
  pausando,
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
        {/* Botão dinâmico: baixar, pausar, continuar */}
        {download.status === "pendente" && (
          <button
            type="button"
            onClick={() => onDownload(download.id)}
            style={{
              background: "#6c63ff",
              color: "#fff",
              border: "none",
              borderRadius: 6,
              padding: "6px 16px",
              fontWeight: 500,
              cursor: "pointer",
              opacity: 1,
              transition: "background 0.2s",
              outline: "none",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              gap: 8,
              fontSize: 16,
              letterSpacing: 0.2,
              minWidth: 90,
            }}
          >
            Baixar
          </button>
        )}
        {download.status === "baixando" && (
          <PauseButton
            type="button"
            onClick={() => onDownload(download.id, "pause")}
            style={{
              display: "flex",
              alignItems: "center",
              gap: 6,
              cursor: pausando ? "not-allowed" : "pointer",
              opacity: pausando ? 0.6 : 1,
              pointerEvents: pausando ? "none" : "auto",
            }}
            disabled={!!pausando}
          >
            {pausando ? (
              <span
                style={{
                  display: "flex",
                  alignItems: "center",
                  marginRight: 6,
                }}
              >
                <ClipLoader size={15} color="#fff" />
              </span>
            ) : (
              <FiPause size={15} style={{ marginRight: 6 }} />
            )}
            Pausar
          </PauseButton>
        )}
        {download.status === "pausado" && (
          <ResumeButton
            type="button"
            onClick={() => onDownload(download.id, "resume")}
            style={{
              display: "flex",
              alignItems: "center",
              gap: 8,
              minWidth: 90,
            }}
          >
            <FiPlay size={15} style={{ marginRight: 6 }} />
            Continuar
          </ResumeButton>
        )}
        {/* Botão abrir pasta */}
        <OpenFolderButton
          type="button"
          onClick={onOpenFolder}
          title="Abrir pasta de downloads"
        >
          Abrir Pasta
        </OpenFolderButton>
        {/* Botão remover */}
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
