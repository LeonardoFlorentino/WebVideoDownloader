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
} from "../../service/downloadsService";
import { baixarVideoTauri } from "../../service/baixarVideo";
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

  // Listener para progresso de download
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      unlisten = await listen<{ url: string; progress: number }>(
        "download_progress",
        async (event) => {
          const { url, progress } = event.payload;
          setDownloads((ds) =>
            ds.map((d) =>
              d.url === url
                ? {
                    ...d,
                    progress: Math.round(progress),
                    status: progress >= 100 ? "concluído" : "baixando",
                  }
                : d,
            ),
          );
          // Se download finalizou, recarrega lista do backend para pegar novo título/status
          if (progress >= 100 && username) {
            try {
              const urls = await getMainUrls(username);
              const baixados = await listDownloadedVideos();
              setDownloads(
                (urls as { url: string; filename: string }[]).map(
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
                      status: baixado ? "concluído" : "pendente",
                      canceled: false,
                    };
                  },
                ),
              );
            } catch (err) {
              console.error("Erro ao atualizar lista após download:", err);
            }
          }
        },
      );
    })();
    return () => {
      if (unlisten) unlisten();
    };
  }, [setDownloads, username]);

  const handleRemove = (id: number) => {
    setDownloads((ds: Download[]) => ds.filter((d: Download) => d.id !== id));
  };

  // Certifique-se de salvar o usuário no localStorage após login/cadastro:
  // localStorage.setItem("lastUser", username);

  const handleDownloadAll = () => {
    setDownloadingAll(true);
    downloads.forEach((d) => {
      if (d.status === "pendente") {
        setDownloads((ds: Download[]) =>
          ds.map((x: Download) =>
            x.id === d.id ? { ...x, status: "baixando", progress: 10 } : x,
          ),
        );
        // Dispara o download sem await
        baixarVideoTauri(undefined, username, d.url, d.filename, d.id)
          .then(() => {
            // O progresso e status serão atualizados via evento
            toast.success(`Download concluído: ${d.filename}`);
          })
          .catch((err) => {
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
          });
      }
    });
    setDownloadingAll(false);
  };

  const handleDownload = async (id: number) => {
    const d = downloads.find((x: Download) => x.id === id);
    if (!d) return;
    // Se o campo filename estiver vazio, buscar o <title> do HTML
    let filenameToUse = d.filename;
    if (
      !filenameToUse ||
      filenameToUse.trim() === "" ||
      filenameToUse.startsWith("video_")
    ) {
      try {
        const title = await getTitleFromUrl(d.url);
        filenameToUse = title;
        // Atualiza no backend
        if (username) {
          await updateMainUrlTitle(username, d.url, d.url, title);
        }
        // Atualiza no frontend
        setDownloads((ds: Download[]) =>
          ds.map((x: Download) =>
            x.id === id ? { ...x, filename: title } : x,
          ),
        );
      } catch {
        // Se não conseguir pegar o título, mantém o nome genérico
        console.warn(
          "Não foi possível obter o título do HTML, usando nome genérico.",
        );
      }
    }
    setDownloads((ds: Download[]) =>
      ds.map((x: Download) =>
        x.id === id ? { ...x, status: "baixando", progress: 10 } : x,
      ),
    );
    // Dispara o download sem await para não travar a UI
    baixarVideoTauri(undefined, username, d.url, d.filename, d.id)
      .then(() => {
        // O status será atualizado pelo evento download_progress
        toast.success(`Download concluído: ${filenameToUse}`);
      })
      .catch((err) => {
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
      });
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
    newVals: { filename: string; url: string; status?: string },
  ) => {
    // Busca o valor original do backend antes de editar
    const original = downloads.find((d) => d.id === id);
    setDownloads((ds: Download[]) =>
      ds.map((d) => {
        if (d.id !== id) return d;
        // Se o usuário clicou em "Marcar como concluído"
        if (newVals.status === "concluído") {
          return { ...d, status: "concluído", progress: 100 };
        }
        // Se a URL mudou, resetar status, progresso e filename
        if (original && newVals.url !== original.url) {
          return {
            ...d,
            url: newVals.url,
            filename: "",
            progress: 0,
            status: "pendente",
          };
        }
        // Se só mudou o nome
        return { ...d, filename: newVals.filename, url: newVals.url };
      }),
    );
    if (username && original && !newVals.status) {
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
        async ({ getMainUrls }) => {
          // Aqui você pode adicionar lógica para salvar no backend se necessário
          // Por simplicidade, apenas recarrega a lista do backend
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
            onEdit={handleEditDownload}
          />
        ))}
      </DownloadList>
    </Container>
  );
}

export default Home;
