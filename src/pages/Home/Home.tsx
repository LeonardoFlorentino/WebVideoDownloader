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
import { getMainUrls, removeMainUrlById } from "../../service/downloadsService";
import type { MainUrl } from "../../service/downloadsService";
import { baixarVideoTauri } from "../../service/baixarVideo";
import { listen } from "@tauri-apps/api/event";
// import { pollDownloadsProgress } from "../../service/downloadsService";

type HomeProps = { username: string };
function Home({ username }: HomeProps) {
  // ...existing code...

  // Novo handleDownload para pause/resume/stop
  const handleDownload = async (id: number, action?: "pause" | "resume") => {
    const d = downloads.find((x: Download) => x.id === id);
    if (!d) return;
    const filename = d.filename.endsWith(".mp4")
      ? d.filename
      : `${d.filename}.mp4`;
    const savePath = `C:/Users/leona/projects/WebVideoDownloader/Vídeos baixados/${filename}`;
    if (action === "pause") {
      setPausando((prev) => ({ ...prev, [d.url]: true }));
      // Não bloqueia a UI aguardando o backend
      import("../../service/pauseDownloadById").then(({ pauseDownloadById }) =>
        pauseDownloadById(d.id, d.url)
          .catch(() => toast.error("Erro ao pausar download"))
          .finally(() => setPausando((prev) => ({ ...prev, [d.url]: false }))));
      return;
    }
    if (action === "resume") {
      setDownloads((ds: Download[]) =>
        ds.map((x: Download) =>
          x.id === id ? { ...x, status: "baixando" } : x,
        ),
      );
      import("../../service/resumeDownload").then(({ resumeDownloadTauri }) => {
        resumeDownloadTauri(String(d.id), username, d.url, savePath).catch(
          () => {
            setDownloads((ds: Download[]) =>
              ds.map((x: Download) =>
                x.id === id ? { ...x, status: "erro", progress: 0 } : x,
              ),
            );
            toast.error("Erro ao retomar download");
          },
        );
      });
      return;
    }
    // Download normal
    try {
      await baixarVideoTauri(String(d.id), username, d.url, savePath);
      setDownloads((ds: Download[]) =>
        ds.map((x: Download) =>
          x.id === id ? { ...x, status: "baixando" } : x,
        ),
      );
    } catch (err) {
      setDownloads((ds: Download[]) =>
        ds.map((x: Download) =>
          x.id === id ? { ...x, status: "erro", progress: 0 } : x,
        ),
      );
      toast.error(`Erro ao baixar vídeo: ${err}`);
    }
  };
  // (Removido: estado e funções do modal de edição não utilizados)
  const [url, setUrl] = useState("");
  const { downloads, setDownloads } = useDownloads() as {
    downloads: Download[];
    setDownloads: React.Dispatch<React.SetStateAction<Download[]>>;
  };
  const [filename, setFilename] = useState("");
  const [downloadingAll, setDownloadingAll] = useState(false);
  const [pausando, setPausando] = useState<{ [url: string]: boolean }>({});
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
          // Para cada url, busca progresso real
          const urlsWithProgress = await Promise.all(
            (urls as MainUrl[]).map(async (item, idx) => {
              // O frontend só deve mostrar "concluído" se vier do backend (getMainUrls)
              // Não deve inferir localmente por progresso ou eventos
              return {
                id: item.id,
                url: item.url,
                filename: item.filename || `video_${idx + 1}`,
                ext: "mp4",
                progress: typeof item.progress === "number" ? item.progress : 0,
                status: item.status || "pendente",
                canceled: false,
                playlist: "", // sempre avulso na Home
              };
            }),
          );
          setDownloads(urlsWithProgress);
        } catch {
          // Removido console.error
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
      unlistenProgress = await listen<{
        id: string;
        progress: number;
        total: number;
        status: string;
      }>("download-progress", (event) => {
        const { id, progress, total, status } = event.payload;
        setDownloads((ds) =>
          ds.map((d) => {
            if (String(d.id) === String(id)) {
              return {
                ...d,
                progress,
                total,
                status:
                  status === "paused"
                    ? "pausado"
                    : status === "completed"
                      ? "concluído"
                      : "baixando",
              };
            }
            return d;
          }),
        );
      });
      unlistenFinished = await listen<{
        url: string;
        filename: string;
        status: string;
        error?: string;
      }>("download_finished", async (event) => {
        const { url, status, error } = event.payload;
        setDownloads((ds) =>
          ds.map((d) => {
            // Só atualiza se:
            // - a url bater
            // - o status atual for 'baixando' (ou seja, só quem estava baixando pode ser concluído)
            if (d.url !== url) return d;
            if (d.status === "pausado") {
              return { ...d, error: error || undefined };
            }
            if (d.status === "baixando") {
              return {
                ...d,
                status: status === "concluido" ? "concluído" : status,
                progress: status === "concluido" ? 100 : d.progress,
                error: error || undefined,
              };
            }
            // Não altera cartões que não estavam baixando
            return d;
          }),
        );
      });
      unlistenPaused = await listen<{ url: string }>(
        "download_paused",
        (event) => {
          const { url } = event.payload;
          setDownloads((ds) =>
            ds.map((d) => (d.url === url ? { ...d, status: "pausado" } : d)),
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

  //

  const handleRemove = async (id: number) => {
    const d = downloads.find((d) => d.id === id);
    if (!d || !username) return;
    try {
      const result = await removeMainUrlById(username, d.id);
      if (result && typeof result === "object" && "ok" in result) {
        const res = result as { ok: boolean; error?: string };
        if (!res.ok && res.error) {
          toast.error(res.error);
          return;
        }
      }
      // Remove apenas o item excluído do estado local
      setDownloads((prev) => prev.filter((d) => d.id !== id));
    } catch (err) {
      let msg = "Erro ao remover vídeo.";
      if (
        err &&
        typeof err === "object" &&
        "error" in err &&
        typeof (err as { error?: string }).error === "string"
      ) {
        msg = (err as { error?: string }).error ?? msg;
      } else if (typeof err === "string") {
        msg = err;
      } else if (err instanceof Error) {
        msg = err.message;
      }
      toast.error(msg);
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
        // Garante extensão .mp4
        const filename = d.filename.endsWith(".mp4")
          ? d.filename
          : `${d.filename}.mp4`;
        baixarVideoTauri(
          String(d.id),
          username,
          d.url,
          `C:/Users/leona/projects/WebVideoDownloader/Vídeos baixados/${filename}`,
        ).catch((err) => {
          setDownloads((ds: Download[]) =>
            ds.map((x: Download) =>
              x.id === d.id ? { ...x, status: "erro", progress: 0 } : x,
            ),
          );
          toast.error(`Erro ao baixar: ${filename}\n${err}`);
        });
      }
    });
    setDownloadingAll(false);
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
      // Garante filename válido
      await import("../../service/downloadsService").then(
        async ({ addMainUrl, getMainUrls }) => {
          // Busca downloads atuais para calcular o próximo idx
          const currentUrls = await getMainUrls(username);
          const idx = Array.isArray(currentUrls) ? currentUrls.length : 0;
          const safeFilename =
            filename && filename.trim() !== ""
              ? filename
              : `video_${idx + 1}.mp4`;
          await addMainUrl(username, url, safeFilename);
          const newUrls = await getMainUrls(username);
          setDownloads(
            (newUrls as (MainUrl & { id?: number; progress?: number })[]).map(
              (item, idx2) => ({
                id: item.id ?? idx2 + 1,
                url: item.url,
                filename: item.filename || `video_${idx2 + 1}`,
                ext: "mp4",
                progress:
                  typeof (item as { progress?: number }).progress === "number"
                    ? (item as { progress?: number }).progress!
                    : 0,
                status: item.status || "pendente",
                canceled: false,
              }),
            ),
          );
          setUrl("");
          setFilename(""); // Limpa o campo de nome após adicionar
        },
      );
    } catch {
      setFilename("");
      // Mensagem de sucesso/erro já é exibida pelo backend, não exibir nada aqui
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
            onOpenFolder={() => openDownloadFolder(d.filename)}
            pausando={!!pausando[d.url]}
          />
        ))}
      </DownloadList>
    </Container>
  );
}
export default Home;
