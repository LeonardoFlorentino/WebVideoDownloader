import { ClipLoader } from "react-spinners";
import React from "react";
import { FiTrash2, FiEdit2, FiPause } from "react-icons/fi";
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
            disabled={download.status === "baixando"}
          >
            <FiEdit2 />
          </button>
        </CardUrlAndStatus>
      </CardTopRow>
      <CardProgressBar
        style={{ position: "relative", height: 24, marginTop: 8 }}
      >
        <CardProgressTrack style={{ height: 24 }}>
          <CardProgressFill
            $status={download.status}
            style={{
              width: `${download.status === "concluído" ? 100 : download.progress}%`,
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
          {download.status === "concluído"
            ? "100.0%"
            : `${typeof download.progress === "number" ? download.progress.toFixed(1) : 0}%`}
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
                  gap: 4,
                }}
              >
                <ClipLoader color="#fff" size={12} speedMultiplier={1.1} />
              </span>
            ) : (
              <FiPause size={15} style={{ verticalAlign: "middle" }} />
            )}
            <span
              style={{ lineHeight: 1, display: "flex", alignItems: "center" }}
            >
              {pausando ? "Pausando..." : "Pausar"}
            </span>
          </PauseButton>
        )}
        {download.status === "pausado" && (
          <button
            type="button"
            onClick={() => onDownload(download.id, "resume")}
            style={{
              background: "#43a047",
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
            Continuar download
          </button>
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
