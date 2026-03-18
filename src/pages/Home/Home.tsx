import { openDownloadFolder } from "../../service/openFolder";
import DownloadCard from "./DownloadCard/DownloadCard";
import React, { useState } from "react";
import { ToastContainer, toast } from "react-toastify";
import "react-toastify/dist/ReactToastify.css";
import {
  Container,
  TopBar,
  NavButton,
  Title,
  UrlForm,
  UrlInput,
  AddButton,
  DownloadList,
  DownloadAllButton,
  EmptyMessage,
  FileNameInput,
  ButtonRow,
} from "./Home.styles";

import { useEffect } from "react";
import { useDownloads } from "../../context/useDownloads";
import type { Download } from "@/types/download";
import { useNavigate } from "react-router-dom";
import {
  baixarEmCascata,
  addMainUrl,
  getMainUrls,
} from "../../service/downloadsService";

function Home() {
  const username = localStorage.getItem("lastUser") || "";
  const [url, setUrl] = useState("");
  const { downloads, setDownloads } = useDownloads() as {
    downloads: Download[];
    setDownloads: React.Dispatch<React.SetStateAction<Download[]>>;
  };
  const [filename, setFilename] = useState("");
  const [downloadingAll, setDownloadingAll] = useState(false);
  const navigate = useNavigate();

  useEffect(() => {
    const fetchUrls = async () => {
      if (username) {
        try {
          const urls = await getMainUrls(username);
          setDownloads(
            (urls as { url: string; filename: string }[]).map((item, idx) => ({
              id: Date.now() + idx,
              url: item.url,
              filename: item.filename || `video_${idx + 1}`,
              ext: "mp4",
              progress: 0,
              status: "pendente",
              canceled: false,
            })),
          );
        } catch (err) {
          // Opcional: mostrar erro ao usuário
          console.error("Erro ao buscar urls do backend:", err);
        }
      }
    };
    fetchUrls();
    window.addEventListener("focus", fetchUrls);
    return () => {
      window.removeEventListener("focus", fetchUrls);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [username]);

  const handleRemove = (id: number) => {
    setDownloads((ds: Download[]) => ds.filter((d: Download) => d.id !== id));
  };

  const handleAdd = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!url.trim()) return;
    setDownloads((prev: Download[]) => [
      {
        id: Date.now(),
        url,
        filename: filename.trim()
          ? filename.trim()
          : `video_${prev.length + 1}`,
        ext: "mp4",
        progress: 0,
        status: "pendente",
        canceled: false,
      },
      ...prev,
    ]);
    setUrl("");
    setFilename("");
    if (username) {
      try {
        await addMainUrl(
          username,
          url,
          filename.trim() ? filename.trim() : undefined,
        );
      } catch (err) {
        console.error("Erro ao salvar url principal no backend:", err);
      }
    }
  };

  const handleDownloadAll = async () => {
    setDownloadingAll(true);
    for (const d of downloads) {
      if (d.status === "pendente") {
        setDownloads((ds: Download[]) =>
          ds.map((x: Download) =>
            x.id === d.id ? { ...x, status: "baixando", progress: 10 } : x,
          ),
        );
        try {
          await baixarEmCascata("Home", [d.url]);
          console.log("SUCESSO: Download realmente baixado!");
          setDownloads((ds: Download[]) =>
            ds.map((x: Download) =>
              x.id === d.id ? { ...x, status: "concluído", progress: 100 } : x,
            ),
          );
          toast.success(`Download concluído: ${d.filename}`);
        } catch (err) {
          console.log("ERRO:", err);
          setDownloads((ds: Download[]) =>
            ds.map((x: Download) =>
              x.id === d.id ? { ...x, status: "erro", progress: 0 } : x,
            ),
          );
          if (typeof err === "string") {
            toast.error(err);
          } else {
            toast.error(`Erro ao baixar: ${d.filename}`);
          }
        }
      }
    }
    setDownloadingAll(false);
  };

  const handleDownload = async (id: number) => {
    const d = downloads.find((x: Download) => x.id === id);
    if (!d) return;
    setDownloads((ds: Download[]) =>
      ds.map((x: Download) =>
        x.id === id ? { ...x, status: "baixando", progress: 10 } : x,
      ),
    );
    try {
      await baixarEmCascata("Home", [d.url]);
      console.log("SUCESSO: Download realmente baixado!");
      setDownloads((ds: Download[]) =>
        ds.map((x: Download) =>
          x.id === id ? { ...x, status: "concluído", progress: 100 } : x,
        ),
      );
      toast.success(`Download concluído: ${d.filename}`);
    } catch (err) {
      console.log("ERRO:", err);
      setDownloads((ds: Download[]) =>
        ds.map((x: Download) =>
          x.id === id ? { ...x, status: "erro", progress: 0 } : x,
        ),
      );
      if (typeof err === "string") {
        toast.error(err);
      } else {
        toast.error(`Erro ao baixar: ${d.filename}`);
      }
    }
  };

  const handleLogout = () => {
    // Aqui você pode limpar o token/autenticação se necessário
    navigate("/login");
  };

  const handleGoToPanel = () => {
    navigate("/user");
  };

  const handleEditDownload = (
    id: number,
    newVals: { filename: string; url: string },
  ) => {
    // Busca o valor original do backend antes de editar
    const original = downloads.find((d) => d.id === id);
    setDownloads((ds: Download[]) =>
      ds.map((d) =>
        d.id === id
          ? { ...d, filename: newVals.filename, url: newVals.url }
          : d,
      ),
    );
    if (username && original) {
      import("../../service/downloadsService").then(
        async ({ updateMainUrlTitle, getMainUrls }) => {
          try {
            await updateMainUrlTitle(
              username,
              original.url,
              newVals.url,
              newVals.filename,
            );
            // Recarrega do backend após editar
            const urls = await getMainUrls(username);
            setDownloads(
              (urls as { url: string; filename: string }[]).map(
                (item, idx) => ({
                  id: Date.now() + idx,
                  url: item.url,
                  filename: item.filename || `video_${idx + 1}`,
                  ext: "mp4",
                  progress: 0,
                  status: "pendente",
                  canceled: false,
                }),
              ),
            );
          } catch (err) {
            console.error("Erro ao atualizar título/url:", err);
            toast.error(
              typeof err === "string"
                ? err
                : "Erro ao salvar edição. Verifique os dados e tente novamente.",
            );
          }
        },
      );
    }
  };

  return (
    <Container>
      <ToastContainer
        position="top-right"
        autoClose={3000}
        hideProgressBar={false}
        newestOnTop
        closeOnClick
        pauseOnFocusLoss
        draggable
        pauseOnHover
      />
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
          onChange={(e) => setUrl(e.target.value)}
        />
        <FileNameInput
          type="text"
          placeholder="Nome do arquivo (opcional)"
          value={filename}
          onChange={(e) => setFilename(e.target.value)}
        />
        <ButtonRow>
          <AddButton type="submit" disabled={!url.trim()}>
            Adicionar
          </AddButton>
          <DownloadAllButton
            onClick={handleDownloadAll}
            disabled={
              downloads.length === 0 ||
              downloads.every((d: Download) => d.status === "concluído") ||
              downloadingAll
            }
          >
            Baixar Todos
          </DownloadAllButton>
        </ButtonRow>
      </UrlForm>
      <DownloadList>
        {downloads.length === 0 && (
          <EmptyMessage>Nenhum vídeo adicionado ainda.</EmptyMessage>
        )}
        {downloads.map((d: Download) => (
          <DownloadCard
            key={d.id}
            download={d}
            onDownload={handleDownload}
            onRemove={handleRemove}
            onOpenFolder={() => openDownloadFolder("Home")}
            onEdit={handleEditDownload}
          />
        ))}
      </DownloadList>
    </Container>
  );
}

export default Home;
