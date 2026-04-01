import React, { useState, useEffect, useRef } from "react";
import { ToastContainer, toast } from "react-toastify";
import "react-toastify/dist/ReactToastify.css";
import { useDownloads } from "../../context/useDownloads";
import type { Download } from "@/types/download";
import { useNavigate } from "react-router-dom";
import { getMainUrls, removeMainUrlById } from "../../service/downloadsService";
import type { MainUrl } from "../../service/downloadsService";
import { baixarVideoTauri } from "../../service/baixarVideo";
import { listen } from "@tauri-apps/api/event";
import { openDownloadFolder } from "../../service/openFolder";
import DownloadCard from "./DownloadCard/DownloadCard";
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

type HomeProps = { username: string };

function normalizeStatus(status?: string): string {
  switch ((status || "").toLowerCase()) {
    case "downloading":
      return "baixando";
    case "completed":
    case "concluido":
      return "concluído";
    case "paused":
      return "pausado";
    case "error":
      return "erro";
    default:
      return status || "pendente";
  }
}

function Home({ username }: HomeProps) {
  const { downloads, setDownloads } = useDownloads() as {
    downloads: Download[];
    setDownloads: React.Dispatch<React.SetStateAction<Download[]>>;
  };

  // Polling eficiente: só ativa quando houver downloads ativos, sem depender de downloads no useEffect
  const pollingRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const hasActiveDownloads = downloads.some(
    (d) =>
      normalizeStatus(d.status) !== "concluído" &&
      normalizeStatus(d.status) !== "erro",
  );
  useEffect(() => {
    if (!username) return;
    // Inicia polling apenas uma vez quando houver downloads ativos
    if (hasActiveDownloads && !pollingRef.current) {
      const poll = async () => {
        const { pollDownloadsProgress } =
          await import("../../service/pollDownloadsProgress");
        setDownloads((prev) => {
          // Checa se ainda há downloads ativos
          const stillActive = prev.some(
            (d) =>
              normalizeStatus(d.status) !== "concluído" &&
              normalizeStatus(d.status) !== "erro",
          );
          if (!stillActive) {
            if (pollingRef.current) {
              clearInterval(pollingRef.current);
              pollingRef.current = null;
            }
            return prev;
          }
          pollDownloadsProgress(prev).then((updated) => {
            setDownloads(updated);
          });
          return prev;
        });
      };
      pollingRef.current = setInterval(() => {
        void poll();
      }, 3000);
      void poll();
    }
    // Cleanup ao desmontar
    return () => {
      if (pollingRef.current) {
        clearInterval(pollingRef.current);
        pollingRef.current = null;
      }
    };
  }, [username, hasActiveDownloads, setDownloads]);
  // ...existing code...

  // Novo handleDownload para pause/resume/stop
  const handleDownload = async (id: number, action?: "pause" | "resume") => {
    const d = downloads.find((x: Download) => x.id === id);
    if (!d) return;
    const fileNameFinal = d.filename.endsWith(".mp4")
      ? d.filename
      : `${d.filename}.mp4`;
    if (action === "pause") {
      setPausando((prev: { [url: string]: boolean }) => ({
        ...prev,
        [d.url]: true,
      }));
      // Atualiza status no front imediatamente para refletir clique do usuário.
      setDownloads((ds: Download[]) =>
        ds.map((x: Download) =>
          x.id === id ? { ...x, status: "pausado" } : x,
        ),
      );
      try {
        const { pauseDownloadById } =
          await import("../../service/pauseDownloadById");
        await pauseDownloadById(d.id, d.url);
      } catch {
        // Se falhar, retorna ao estado anterior para não mascarar erro real.
        setDownloads((ds: Download[]) =>
          ds.map((x: Download) =>
            x.id === id ? { ...x, status: d.status } : x,
          ),
        );
        toast.error("Erro ao pausar download");
      } finally {
        setPausando((prev: { [url: string]: boolean }) => ({
          ...prev,
          [d.url]: false,
        }));
      }
      return;
    }
    if (action === "resume") {
      const savePath = `C:/Users/leona/projects/WebVideoDownloader/Vídeos baixados/${fileNameFinal}`;
      // Restaura o progresso anterior (não reseta para 0%)
      setDownloads((ds: Download[]) =>
        ds.map((x: Download) =>
          x.id === id
            ? { ...x, status: "baixando", progress: x.progress, total: x.total }
            : x,
        ),
      );
      import("../../service/resumeDownload").then(({ resumeDownloadTauri }) => {
        resumeDownloadTauri(String(d.id), username, d.url, savePath).catch(
          (err: unknown) => {
            setDownloads((ds: Download[]) =>
              ds.map((x: Download) =>
                x.id === id ? { ...x, status: "pausado" } : x,
              ),
            );
            const msg = err instanceof Error ? err.message : String(err);
            toast.error(`Erro ao retomar download: ${msg}`);
          },
        );
      });
      return;
    }
    // Download normal
    try {
      const savePath = `C:/Users/leona/projects/WebVideoDownloader/Vídeos baixados/${fileNameFinal}`;
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
  // (Removido: estado e funções do modal de edição não utilizados)
  const [url, setUrl] = useState("");
  const [filename, setFilename] = useState("");
  const [downloadingAll, setDownloadingAll] = useState(false);
  const [pausando, setPausando] = useState<{ [url: string]: boolean }>({});
  const navigate = useNavigate();
  const downloadsRef = useRef<Download[]>(downloads);

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
          const urlsWithProgress = await Promise.all(
            (urls as MainUrl[]).map(async (item, idx) => {
              return {
                id: item.id,
                url: item.url,
                filename: item.filename || `video_${idx + 1}`,
                ext: "mp4",
                progress: typeof item.progress === "number" ? item.progress : 0,
                status: normalizeStatus(item.status),
                canceled: false,
                playlist: "",
              };
            }),
          );
          setDownloads(urlsWithProgress);
        } catch {
          // erro silencioso
        }
      }
    };
    fetchUrls();
    window.addEventListener("focus", fetchUrls);
    return () => {
      window.removeEventListener("focus", fetchUrls);
    };
    //
  }, [username, setDownloads]);

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
                status: normalizeStatus(status),
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
            if (normalizeStatus(d.status) === "pausado") {
              return { ...d, error: error || undefined };
            }
            if (normalizeStatus(d.status) === "baixando") {
              return {
                ...d,
                status: normalizeStatus(status),
                progress:
                  normalizeStatus(status) === "concluído" ? 100 : d.progress,
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
    const d = downloads.find((d: Download) => d.id === id);
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
      setDownloads((prev: Download[]) =>
        prev.filter((d: Download) => d.id !== id),
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
    downloads.forEach((d: Download) => {
      if (d.status === "pendente") {
        const filenameFinal = d.filename.endsWith(".mp4")
          ? d.filename
          : `${d.filename}.mp4`;
        baixarVideoTauri(
          String(d.id),
          username,
          d.url,
          `C:/Users/leona/projects/WebVideoDownloader/Vídeos baixados/${filenameFinal}`,
        ).catch((err: unknown) => {
          setDownloads((ds: Download[]) =>
            ds.map((x: Download) =>
              x.id === d.id ? { ...x, status: "erro", progress: 0 } : x,
            ),
          );
          toast.error(`Erro ao baixar: ${filenameFinal}\n${err}`);
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
            onOpenFolder={() => openDownloadFolder("")}
            pausando={!!pausando[d.url]}
          />
        ))}
      </DownloadList>
    </Container>
  );
}
export default Home;
