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

import { useEffect, useRef } from "react";
import { useDownloads } from "../../context/useDownloads";
import type { Download } from "@/types/download";
import { useNavigate } from "react-router-dom";
import {
  getMainUrls,
  getTitleFromUrl,
  updateMainUrlTitle,
  removeMainUrl,
} from "../../service/downloadsService";
import { baixarVideoTauri } from "../../service/baixarVideo";
import { pausarDownloadTauri } from "../../service/pausarDownload";
import { listen } from "@tauri-apps/api/event";
import { listDownloadedVideos } from "../../lib/listVideos";

type HomeProps = { username: string };
function Home({ username }: HomeProps) {
  const [url, setUrl] = useState("");
  const { downloads, setDownloads } = useDownloads() as {
    downloads: Download[];
    setDownloads: React.Dispatch<React.SetStateAction<Download[]>>;
  };
  const [filename, setFilename] = useState("");
  const [downloadingAll, setDownloadingAll] = useState(false);
  const navigate = useNavigate();
  const downloadsRef = useRef(downloads);

  // Redireciona para login se username estiver vazio
  useEffect(() => {
    if (!username) {
      navigate("/login");
    }
  }, [username, navigate]);
  useEffect(() => {
    downloadsRef.current = downloads;
  }, [downloads]);

  useEffect(() => {
    const fetchUrls = async () => {
      if (username) {
        try {
          const urls = await getMainUrls(username);
          // Verifica vídeos já baixados
          const baixados = await listDownloadedVideos();
          setDownloads(
            (urls as { url: string; filename: string; status?: string }[]).map(
              (item, idx) => {
                const nomeSemExt = item.filename.replace(/\.[^/.]+$/, "");
                const baixado = baixados.find((b) => {
                  const nomeSemExtNorm = nomeSemExt
                    .toLowerCase()
                    .normalize("NFD")
                    .replace(/[^\w\s]/g, "");
                  const baixadoSemExtNorm = b.name
                    .replace(/\.[^/.]+$/, "")
                    .toLowerCase()
                    .normalize("NFD")
                    .replace(/[^\w\s]/g, "");
                  return baixadoSemExtNorm === nomeSemExtNorm;
                });
                return {
                  id: Date.now() + idx,
                  url: item.url,
                  filename: item.filename || `video_${idx + 1}`,
                  ext: "mp4",
                  progress: baixado ? 100 : 0,
                  status: item.status || (baixado ? "concluído" : "pendente"),
                  canceled: false,
                  playlist: "", // sempre avulso na Home
                };
              },
            ),
          );
        } catch (err) {
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

  // Listeners para progresso, finalização e pausa de download
  useEffect(() => {
    let unlistenProgress: (() => void) | undefined;
    let unlistenFinished: (() => void) | undefined;
    let unlistenPaused: (() => void) | undefined;
    (async () => {
      unlistenProgress = await listen<{ url: string; progress: number }>(
        "download_progress",
        async (event) => {
          const { url, progress } = event.payload;
          setDownloads((ds) =>
            ds.map((d) =>
              d.url === url
                ? {
                    ...d,
                    progress: progress,
                    status: progress >= 100 ? "concluído" : "baixando",
                  }
                : d,
            ),
          );
        },
      );
      unlistenFinished = await listen<{
        url: string;
        filename: string;
        status: string;
        error?: string;
      }>("download_finished", async (event) => {
        const { url, status, error } = event.payload;
        setDownloads((ds) =>
          ds.map((d) =>
            d.url === url
              ? {
                  ...d,
                  status: status === "concluido" ? "concluído" : status,
                  progress: status === "concluido" ? 100 : d.progress,
                  error: error || undefined,
                }
              : d,
          ),
        );
      });
      unlistenPaused = await listen<{ url: string }>(
        "download_paused",
        (event) => {
          const { url } = event.payload;
          setDownloads((ds) =>
            ds.map((d) => (d.url === url ? { ...d, status: "pendente" } : d)),
          );
        },
      );
    })();
    return () => {
      if (unlistenProgress) unlistenProgress();
      if (unlistenFinished) unlistenFinished();
      if (unlistenPaused) unlistenPaused();
    };
  }, [setDownloads]);

  const handleRemove = async (id: number) => {
    const d = downloads.find((d) => d.id === id);
    if (!d || !username) return;
    try {
      await removeMainUrl(username, d.url);
      // Recarrega a lista do backend após remover
      const urls = await getMainUrls(username);
      const baixados = await import("../../lib/listVideos").then((m) =>
        m.listDownloadedVideos(),
      );
      setDownloads(
        (urls as { url: string; filename: string; status?: string }[]).map(
          (item, idx) => {
            const nomeSemExt = item.filename.replace(/\.[^/.]+$/, "");
            const baixado = baixados.find((b: { name: string }) => {
              const nomeSemExtNorm = nomeSemExt
                .toLowerCase()
                .normalize("NFD")
                .replace(/[^\w\s]/g, "");
              const baixadoSemExtNorm = b.name
                .replace(/\.[^/.]+$/, "")
                .toLowerCase()
                .normalize("NFD")
                .replace(/[^\w\s]/g, "");
              return baixadoSemExtNorm === nomeSemExtNorm;
            });
            return {
              id: Date.now() + idx,
              url: item.url,
              filename: item.filename || `video_${idx + 1}`,
              ext: "mp4",
              progress: baixado ? 100 : 0,
              status: item.status || (baixado ? "concluído" : "pendente"),
              canceled: false,
            };
          },
        ),
      );
      toast.success("Vídeo removido!");
    } catch {
      toast.error("Erro ao remover vídeo!");
    }
  };

  // Certifique-se de salvar o usuário no localStorage após login/cadastro:
  // localStorage.setItem("lastUser", username);

  const handleDownloadAll = () => {
    setDownloadingAll(true);
    downloads.forEach((d) => {
      if (d.status === "pendente") {
        setDownloads((ds: Download[]) =>
          ds.map((x: Download) =>
            x.id === d.id ? { ...x, status: "baixando" } : x,
          ),
        );
        // Dispara o download sem await
        baixarVideoTauri(undefined, username, d.url, d.filename, d.id).catch(
          (err) => {
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
          },
        );
      }
    });
    setDownloadingAll(false);
  };

  // Novo handleDownload para pause/resume/stop
  const handleDownload = async (id: number, action?: "pause" | "resume") => {
    const d = downloads.find((x: Download) => x.id === id);
    if (!d) return;
    if (action === "pause") {
      if (d) await pausarDownloadTauri(d.url);
      return;
    }
    if (action === "resume") {
      setDownloads((ds: Download[]) =>
        ds.map((x: Download) =>
          x.id === id ? { ...x, status: "baixando" } : x,
        ),
      );
      // Retoma o download (chama baixarVideoTauri novamente)
      baixarVideoTauri(undefined, username, d.url, d.filename, d.id).catch(
        (err) => {
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
        },
      );
      return;
    }
    // Download normal
    let filenameToUse = d.filename;
    if (
      !filenameToUse ||
      filenameToUse.trim() === "" ||
      filenameToUse.startsWith("video_")
    ) {
      try {
        const title = await getTitleFromUrl(d.url);
        filenameToUse = title;
        if (username) {
          await updateMainUrlTitle(username, d.url, d.url, title);
        }
        setDownloads((ds: Download[]) =>
          ds.map((x: Download) =>
            x.id === id ? { ...x, filename: title } : x,
          ),
        );
      } catch {
        console.warn(
          "Não foi possível obter o título do HTML, usando nome genérico.",
        );
      }
    }
    setDownloads((ds: Download[]) =>
      ds.map((x: Download) => (x.id === id ? { ...x, status: "baixando" } : x)),
    );
    baixarVideoTauri(undefined, username, d.url, d.filename, d.id).catch(
      (err) => {
        setDownloads((ds: Download[]) =>
          ds.map((x: Download) =>
            x.id === id ? { ...x, status: "erro", progress: 0 } : x,
          ),
        );
        if (typeof err === "string") {
          toast.error(err);
        } else {
          toast.error(`Erro ao baixar: ${filenameToUse}`);
        }
      },
    );
  };

  const handleLogout = () => {
    // Aqui você pode limpar o token/autenticação se necessário
    navigate("/login");
  };

  const handleGoToPanel = () => {
    navigate("/user");
  };

  // Função para adicionar novo download
  const handleAdd = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!url.trim()) return;
    if (!username) {
      toast.error("Usuário não identificado. Faça login novamente.");
      navigate("/login");
      return;
    }
    try {
      // Chama o backend para adicionar a URL
      await import("../../service/downloadsService").then(
        async ({ addMainUrl, getMainUrls }) => {
          await addMainUrl(username, url, filename);
          const urls = await getMainUrls(username);
          const baixados = await listDownloadedVideos();
          setDownloads(
            (urls as { url: string; filename: string; status?: string }[]).map(
              (item, idx) => {
                const nomeSemExt = item.filename.replace(/\.[^/.]+$/, "");
                const baixado = baixados.find((b) => {
                  const nomeSemExtNorm = nomeSemExt
                    .toLowerCase()
                    .normalize("NFD")
                    .replace(/[^\w\s]/g, "");
                  const baixadoSemExtNorm = b.name
                    .replace(/\.[^/.]+$/, "")
                    .toLowerCase()
                    .normalize("NFD")
                    .replace(/[^\w\s]/g, "");
                  return baixadoSemExtNorm === nomeSemExtNorm;
                });
                return {
                  id: Date.now() + idx,
                  url: item.url,
                  filename: item.filename || `video_${idx + 1}`,
                  ext: "mp4",
                  progress: baixado ? 100 : 0,
                  status: item.status || (baixado ? "concluído" : "pendente"),
                  canceled: false,
                };
              },
            ),
          );
        },
      );
      setUrl("");
      setFilename("");
      toast.success("Vídeo adicionado!");
    } catch (err) {
      console.error("Erro ao adicionar download:", err);
      toast.error("Erro ao adicionar download. Tente novamente.");
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
            onOpenFolder={() =>
              openDownloadFolder(d.playlist ? d.playlist : "")
            }
          />
        ))}
      </DownloadList>
    </Container>
  );
}

export default Home;
