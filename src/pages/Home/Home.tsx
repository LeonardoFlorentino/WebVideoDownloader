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
// import { pausarDownloadTauri } from "../../service/pausarDownload";
import { resumeDownloadTauri } from "../../service/resumeDownload";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { listDownloadedVideos } from "../../lib/listVideos";
import { pollDownloadsProgress } from "../../service/downloadsService";

type HomeProps = { username: string };
function Home({ username }: HomeProps) {
  const [url, setUrl] = useState("");
  const { downloads, setDownloads } = useDownloads() as {
    downloads: Download[];
    setDownloads: React.Dispatch<React.SetStateAction<Download[]>>;
  };
  const [filename, setFilename] = useState("");
  const [downloadingAll, setDownloadingAll] = useState(false);
  const [pausando, setPausando] = useState<{ [url: string]: boolean }>({});
  const [pollingAtivo, setPollingAtivo] = useState(false);
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
            if (d.url !== url) return d;
            // Se o status atual for 'pausado', não sobrescreva para 'concluído'
            if (d.status === "pausado") {
              return { ...d, error: error || undefined };
            }
            return {
              ...d,
              status: status === "concluido" ? "concluído" : status,
              progress: status === "concluido" ? 100 : d.progress,
              error: error || undefined,
            };
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
          setPausando((prev) => ({ ...prev, [url]: false }));
        },
      );
    })();
    return () => {
      if (unlistenProgress) unlistenProgress();
      if (unlistenFinished) unlistenFinished();
      if (unlistenPaused) unlistenPaused();
    };
  }, [setDownloads]);

  // Polling de progresso dos downloads ativos (só após baixar)
  useEffect(() => {
    if (!pollingAtivo) return;
    const interval = setInterval(async () => {
      // Só faz polling se houver downloads ativos
      const ativos = downloads.filter(
        (d) => d.status === "baixando" || d.status === "pendente",
      );
      if (ativos.length === 0) {
        // Se todos os downloads estão concluídos, encerra o polling
        const statusFinalizados = [
          "concluído",
          "concluido",
          "sucesso",
          "finalizado",
        ];
        const todosConcluidos =
          downloads.length > 0 &&
          downloads.every((d) =>
            statusFinalizados.includes((d.status || "").toLowerCase()),
          );
        if (todosConcluidos) {
          setPollingAtivo(false);
        }
        return;
      }
      const progressos = await pollDownloadsProgress(ativos);
      setDownloads((ds) =>
        ds.map((d) => {
          const found = progressos.find((p) => p.id === d.id);
          if (
            found &&
            typeof found.progress === "number" &&
            typeof found.total === "number"
          ) {
            // Salva progress e total como bytes, não percentual!
            return {
              ...d,
              progress: found.progress,
              total: found.total,
              status: found.status || d.status,
            };
          }
          return d;
        }),
      );
    }, 1000);
    return () => clearInterval(interval);
  }, [downloads, setDownloads, pollingAtivo]);

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
      // Recarrega a lista do backend após remover
      const urls = await getMainUrls(username);
      const baixados = await import("../../lib/listVideos").then((m) =>
        m.listDownloadedVideos(),
      );
      setDownloads(
        (urls as MainUrl[]).map((item, idx) => {
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
          const status = baixado ? "concluído" : "pendente";
          const progress = baixado ? 100 : 0;
          return {
            id: item.id,
            url: item.url,
            filename: item.filename || `video_${idx + 1}`,
            ext: "mp4",
            progress,
            status,
            canceled: false,
          };
        }),
      );
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
    setPollingAtivo(true); // Ativa polling/logs ao baixar
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

  // Novo handleDownload para pause/resume/stop
  const handleDownload = async (id: number, action?: "pause" | "resume") => {
    setPollingAtivo(true); // Ativa polling/logs ao baixar individual
    const d = downloads.find((x: Download) => x.id === id);
    if (!d) return;
    const filename = d.filename.endsWith(".mp4")
      ? d.filename
      : `${d.filename}.mp4`;
    const savePath = `C:/Users/leona/projects/WebVideoDownloader/Vídeos baixados/${filename}`;
    if (action === "pause") {
      setPausando((prev) => ({ ...prev, [d.url]: true }));
      try {
        await invoke("integrated_pause_download", {
          id: String(d.id),
          url: d.url,
        });
        setPausando((prev) => ({ ...prev, [d.url]: false }));
      } catch {
        setPausando((prev) => ({ ...prev, [d.url]: false }));
        toast.error("Erro ao pausar download");
      }
      return;
    }
    if (action === "resume") {
      setDownloads((ds: Download[]) =>
        ds.map((x: Download) =>
          x.id === id ? { ...x, status: "baixando" } : x,
        ),
      );
      resumeDownloadTauri(String(d.id), username, d.url, savePath).catch(() => {
        setDownloads((ds: Download[]) =>
          ds.map((x: Download) =>
            x.id === id ? { ...x, status: "erro", progress: 0 } : x,
          ),
        );
        toast.error("Erro ao retomar download");
      });
      return;
    }
    // Download normal
    setDownloads((ds: Download[]) =>
      ds.map((x: Download) => (x.id === id ? { ...x, status: "baixando" } : x)),
    );
    try {
      await baixarVideoTauri(String(d.id), username, d.url, savePath);
    } catch (err) {
      setDownloads((ds: Download[]) =>
        ds.map((x: Download) =>
          x.id === id ? { ...x, status: "erro", progress: 0 } : x,
        ),
      );
      toast.error(`Erro ao baixar vídeo: ${err}`);
    }
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
          const baixados = await listDownloadedVideos();
          setDownloads(
            (
              newUrls as { url: string; filename: string; status?: string }[]
            ).map((item, idx2) => {
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
                id: Date.now() + idx2,
                url: item.url,
                filename: item.filename || `video_${idx2 + 1}`,
                ext: "mp4",
                progress: baixado ? 100 : 0,
                status: item.status || (baixado ? "concluído" : "pendente"),
                canceled: false,
              };
            }),
          );
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
            onOpenFolder={() => openDownloadFolder("")}
            pausando={!!pausando[d.url]}
          />
        ))}
      </DownloadList>
    </Container>
  );
}
export default Home;
