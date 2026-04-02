import React from "react";
import { invoke } from "@tauri-apps/api/core";
import { FiDownload, FiPause, FiPlay, FiFolder } from "react-icons/fi";
import { ToastContainer, toast } from "react-toastify";
import "react-toastify/dist/ReactToastify.css";
import { useNavigate } from "react-router-dom";
import {
  Container,
  Card,
  Header,
  IconCircle,
  Username,
  TopBarFixed,
  WelcomeWrapper,
  CreatePlaylistWrapper,
  PlaylistsWrapper,
  PlaylistCard,
  PlaylistCardTitle,
  PlaylistLinksLabel,
  PlaylistLinksList,
  PlaylistLinkItem,
  PlaylistLinkIndex,
  PlaylistLinkContent,
  PlaylistUrlText,
  PlaylistFilenameText,
  ProgressTrack,
  ProgressFill,
  ProgressMeta,
  DownloadingBadge,
  PlaylistActionsRow,
  CascadeIndicator,
  NoPlaylists,
  ModalOverlay,
  ModalForm,
  ModalTitle,
  ModalInput,
  ModalLinksLabel,
  ModalLinkFieldWrapper,
  ModalLinkInputsColumn,
  ModalLinkButtonsColumn,
  ModalLinkInput,
  ModalButton,
  ModalButtonAdd,
  ModalActions,
  WelcomeHeaderRow,
  PlaylistCardHeader,
  EditButton,
  DeleteButton,
  CreateButton,
  SubmitButton,
  NoLinkItem,
  CancelButton,
  ConfirmModal,
  ConfirmTitle,
  ConfirmText,
  ConfirmActions,
  ConfirmCancelButton,
  ConfirmDeleteButton,
} from "./UserPanel.styles";
import { NavButton } from "../Home/Home.styles";
import type { Playlist } from "@/types/playlist";
import { openDownloadFolder } from "../../service/openFolder";
import { baixarVideoTauri } from "../../service/baixarVideo";
import { resumeDownloadTauri } from "../../service/resumeDownload";
import {
  getPanelPlaylists,
  replacePanelPlaylists,
} from "../../service/panelPlaylistsService";

import { useDownloads } from "../../context/useDownloads";

type LinkDownloadState = {
  status: string;
  progress: number;
  total: number;
};

type ProgressCommandResult = {
  ok: boolean;
  data?: { downloaded?: number; total_size?: number; status?: string };
  error?: string;
};

type CascadeControlState = {
  index: number;
  downloading: boolean;
  paused: boolean;
  currentId?: string;
};

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

const UserPanel: React.FC = () => {
  const navigate = useNavigate();
  const { username } = useDownloads();
  const [playlists, setPlaylists] = React.useState<Playlist[]>([]);

  React.useEffect(() => {
    if (!username) {
      setPlaylists([]);
      return;
    }

    let active = true;

    const loadPlaylists = async () => {
      try {
        const backendPlaylists = await getPanelPlaylists(username);
        if (!active) {
          return;
        }

        if (backendPlaylists.length > 0) {
          setPlaylists(backendPlaylists);
          return;
        }

        const legacyData = localStorage.getItem(`playlists_${username}`);
        const legacyPlaylists = legacyData
          ? (JSON.parse(legacyData) as Playlist[])
          : [];

        if (legacyPlaylists.length > 0) {
          await replacePanelPlaylists(username, legacyPlaylists);
          if (active) {
            setPlaylists(legacyPlaylists);
          }
          localStorage.removeItem(`playlists_${username}`);
          return;
        }

        setPlaylists([]);
      } catch {
        if (active) {
          setPlaylists([]);
        }
      }
    };

    void loadPlaylists();

    return () => {
      active = false;
    };
  }, [username]);
  const [showModal, setShowModal] = React.useState<boolean>(false);
  const [modalTitle, setModalTitle] = React.useState<string>("");
  const [modalLinks, setModalLinks] = React.useState<
    { url: string; filename: string }[]
  >([{ url: "", filename: "" }]);
  const [editId, setEditId] = React.useState<string | null>(null);
  const [deleteConfirmId, setDeleteConfirmId] = React.useState<string | null>(
    null,
  );
  const [cascadeState, setCascadeState] = React.useState<{
    [playlistId: string]: CascadeControlState;
  }>({});
  const [linkProgress, setLinkProgress] = React.useState<{
    [key: string]: LinkDownloadState;
  }>({});
  const cascadeStateRef = React.useRef(cascadeState);
  const resumeGuardRef = React.useRef<{ [key: string]: number }>({});
  React.useEffect(() => {
    cascadeStateRef.current = cascadeState;
  }, [cascadeState]);

  const updateCascadeForPlaylist = React.useCallback(
    (
      playlistId: string,
      updater: (prev: CascadeControlState | undefined) => CascadeControlState,
    ) => {
      setCascadeState((prev) => {
        const nextForPlaylist = updater(prev[playlistId]);
        const next = {
          ...prev,
          [playlistId]: nextForPlaylist,
        };
        cascadeStateRef.current = next;
        return next;
      });
    },
    [],
  );

  const syncPlaylistProgress = React.useCallback(async () => {
    const updates: { [key: string]: LinkDownloadState } = {};

    await Promise.all(
      playlists.flatMap((playlist) =>
        playlist.links.map(async (link, idx) => {
          try {
            const result = (await invoke("get_progress_command", {
              url: link.url,
              progressKey: progressKeyForLink(playlist.id, link.id),
            })) as {
              ok: boolean;
              data?: {
                downloaded?: number;
                total_size?: number;
                status?: string;
              };
            };

            if (result?.ok && result.data) {
              const key = linkKey(playlist.id, idx);
              const normalizedStatus = normalizeStatus(result.data.status);
              const guardUntil = resumeGuardRef.current[key] || 0;
              const guardedStatus =
                normalizedStatus === "pausado" && Date.now() < guardUntil
                  ? "baixando"
                  : normalizedStatus;

              if (normalizedStatus !== "pausado" && guardUntil > 0) {
                delete resumeGuardRef.current[key];
              }

              updates[linkKey(playlist.id, idx)] = {
                status: guardedStatus,
                progress: Number(result.data.downloaded || 0),
                total: Number(result.data.total_size || 0),
              };
            }
          } catch {
            updates[linkKey(playlist.id, idx)] = {
              status: "pendente",
              progress: 0,
              total: 0,
            };
          }
        }),
      ),
    );

    setLinkProgress((prev) => ({ ...prev, ...updates }));

    setCascadeState((prev) => {
      const next = { ...prev };

      for (const playlist of playlists) {
        const statuses = playlist.links.map((_, idx) => {
          const state = updates[linkKey(playlist.id, idx)];
          return state?.status || "pendente";
        });

        const activeIndex = statuses.findIndex(
          (status) =>
            status === "baixando" ||
            status === "preparando" ||
            status === "convertendo",
        );
        const pausedIndex = statuses.findIndex(
          (status) => status === "pausado",
        );

        if (activeIndex >= 0) {
          next[playlist.id] = {
            index: activeIndex,
            downloading: true,
            paused: false,
            currentId: prev[playlist.id]?.currentId,
          };
          continue;
        }

        if (pausedIndex >= 0) {
          next[playlist.id] = {
            index: pausedIndex,
            downloading: false,
            paused: true,
            currentId: prev[playlist.id]?.currentId,
          };
          continue;
        }

        const firstPendingIndex = statuses.findIndex(
          (status) => status !== "concluído" && status !== "erro",
        );

        next[playlist.id] = {
          index:
            firstPendingIndex >= 0 ? firstPendingIndex : playlist.links.length,
          downloading: false,
          paused: false,
          currentId: prev[playlist.id]?.currentId,
        };
      }

      return next;
    });
  }, [playlists]);

  React.useEffect(() => {
    void syncPlaylistProgress();
  }, [syncPlaylistProgress]);

  React.useEffect(() => {
    const hasTrackedLinks = playlists.some(
      (playlist) => playlist.links.length > 0,
    );
    if (!hasTrackedLinks) return;

    const timer = window.setInterval(() => {
      void syncPlaylistProgress();
    }, 3000);

    return () => {
      window.clearInterval(timer);
    };
  }, [playlists, syncPlaylistProgress]);

  const linkKey = (playlistId: string, idx: number) => `${playlistId}:${idx}`;
  const progressKeyForLink = (playlistId: string, linkId: string) =>
    `panel:${username || "anon"}:${playlistId}:${linkId}`;

  const savePathForLink = (
    playlistName: string,
    idx: number,
    link: { url: string; filename?: string },
  ) => {
    const safeName = (link.filename || `video_${idx + 1}`).trim();
    const filenameFinal = safeName.endsWith(".mp4")
      ? safeName
      : `${safeName}.mp4`;
    return `C:/Users/leona/projects/WebVideoDownloader/Vídeos baixados/${playlistName}/${filenameFinal}`;
  };

  const syncLinkWithDisk = async (
    playlistId: string,
    playlistName: string,
    idx: number,
    link: { id: string; url: string; filename?: string },
  ): Promise<LinkDownloadState> => {
    const result = (await invoke("sync_download_file_state_command", {
      url: link.url,
      savePath: savePathForLink(playlistName, idx, link),
      progressKey: progressKeyForLink(playlistId, link.id),
      source: "panel",
    })) as ProgressCommandResult;

    const nextState = {
      status: normalizeStatus(result?.data?.status),
      progress: Number(result?.data?.downloaded || 0),
      total: Number(result?.data?.total_size || 0),
    };

    setLinkProgress((prev) => ({
      ...prev,
      [linkKey(playlistId, idx)]: nextState,
    }));

    return nextState;
  };

  const waitForTerminalStatus = async (
    playlistId: string,
    idx: number,
    linkId: string,
    url: string,
  ): Promise<string> => {
    while (true) {
      try {
        const result = (await invoke("get_progress_command", {
          url,
          progressKey: progressKeyForLink(playlistId, linkId),
        })) as {
          ok: boolean;
          data?: { downloaded?: number; total_size?: number; status?: string };
        };
        const rawStatus = result?.data?.status || "preparando";
        const key = linkKey(playlistId, idx);
        const normalizedStatus = normalizeStatus(rawStatus);
        const guardUntil = resumeGuardRef.current[key] || 0;
        const status =
          normalizedStatus === "pausado" && Date.now() < guardUntil
            ? "baixando"
            : normalizedStatus;
        const downloaded = Number(result?.data?.downloaded || 0);
        const total = Number(result?.data?.total_size || 0);
        setLinkProgress((prev) => ({
          ...prev,
          [key]: { status, progress: downloaded, total },
        }));

        if (normalizedStatus !== "pausado" && guardUntil > 0) {
          delete resumeGuardRef.current[key];
        }

        if (
          status === "concluído" ||
          status === "erro" ||
          status === "pausado"
        ) {
          return status;
        }
      } catch {
        // polling resiliente: tenta novamente
      }
      await new Promise((r) => setTimeout(r, 1000));
    }
  };

  const processCascadeFrom = async (
    playlistId: string,
    links: { id: string; url: string; filename?: string }[],
    playlistName: string,
    startIndex: number,
    resumeCurrent: boolean,
  ) => {
    for (let idx = startIndex; idx < links.length; idx += 1) {
      const latest = cascadeStateRef.current[playlistId];
      if (latest?.paused && !(resumeCurrent && idx === startIndex)) {
        updateCascadeForPlaylist(playlistId, (prev) => ({
          ...(prev || { index: idx, downloading: false, paused: true }),
          downloading: false,
          paused: true,
          index: idx,
        }));
        return;
      }

      const link = links[idx];
      if (!resumeCurrent) {
        const diskState = await syncLinkWithDisk(
          playlistId,
          playlistName,
          idx,
          link,
        );
        if (diskState.status === "concluído") {
          updateCascadeForPlaylist(playlistId, (prev) => ({
            ...(prev || { index: idx + 1, downloading: false, paused: false }),
            index: idx + 1,
            downloading: idx + 1 < links.length,
            paused: false,
          }));
          continue;
        }
      }

      const downloadId = latest?.currentId || `${Date.now()}_${idx}`;
      const savePath = savePathForLink(playlistName, idx, link);

      updateCascadeForPlaylist(playlistId, () => ({
        index: idx,
        downloading: true,
        paused: false,
        currentId: downloadId,
      }));

      try {
        if (resumeCurrent && idx === startIndex) {
          resumeGuardRef.current[linkKey(playlistId, idx)] = Date.now() + 30000;
          setLinkProgress((prev) => ({
            ...prev,
            [linkKey(playlistId, idx)]: {
              status: "baixando",
              progress: prev[linkKey(playlistId, idx)]?.progress || 0,
              total: prev[linkKey(playlistId, idx)]?.total || 0,
            },
          }));
          await resumeDownloadTauri(
            downloadId,
            username || "",
            link.url,
            savePath,
            {
              progressKey: progressKeyForLink(playlistId, link.id),
              source: "panel",
            },
          );
        } else {
          await baixarVideoTauri(
            downloadId,
            username || "",
            link.url,
            savePath,
            {
              progressKey: progressKeyForLink(playlistId, link.id),
              source: "panel",
            },
          );
        }
      } catch (err) {
        setLinkProgress((prev) => ({
          ...prev,
          [linkKey(playlistId, idx)]: {
            status: "erro",
            progress: prev[linkKey(playlistId, idx)]?.progress || 0,
            total: prev[linkKey(playlistId, idx)]?.total || 0,
          },
        }));
        toast.error(
          resumeCurrent && idx === startIndex
            ? `Erro ao continuar download: ${err}`
            : `Erro ao iniciar download: ${err}`,
        );
        continue;
      }

      const finalStatus = await waitForTerminalStatus(
        playlistId,
        idx,
        link.id,
        link.url,
      );
      if (finalStatus === "pausado") {
        updateCascadeForPlaylist(playlistId, (prev) => ({
          ...(prev || { index: idx, downloading: false, paused: true }),
          downloading: false,
          paused: true,
          index: idx,
        }));
        return;
      }

      updateCascadeForPlaylist(playlistId, (prev) => ({
        ...(prev || { index: idx + 1, downloading: false, paused: false }),
        index: idx + 1,
        downloading: idx + 1 < links.length,
        paused: false,
      }));
    }

    updateCascadeForPlaylist(playlistId, (prev) => ({
      ...(prev || { index: links.length, downloading: false, paused: false }),
      index: links.length,
      downloading: false,
      paused: false,
    }));
    toast.success("Download em cascata finalizado!", {
      position: "top-center",
      autoClose: 2000,
    });
  };

  const handleCascadeDownload = async (
    playlistId: string,
    links: { id: string; url: string; filename?: string }[],
    playlistName: string,
  ) => {
    if (!links.length) return;
    try {
      await invoke("create_download_folder_tauri", { playlist: playlistName });
    } catch (err) {
      toast.error(`Erro ao criar pasta da playlist: ${err}`);
      return;
    }

    const reconciledStates = await Promise.all(
      links.map((link, idx) =>
        syncLinkWithDisk(playlistId, playlistName, idx, link),
      ),
    );
    const startIndex = reconciledStates.findIndex(
      (state) => state.status !== "concluído",
    );

    if (startIndex === -1) {
      updateCascadeForPlaylist(playlistId, () => ({
        index: links.length,
        downloading: false,
        paused: false,
      }));
      toast.info("Todos os vídeos da playlist já existem na pasta.");
      return;
    }

    updateCascadeForPlaylist(playlistId, () => ({
      index: startIndex,
      downloading: true,
      paused: false,
    }));
    await processCascadeFrom(
      playlistId,
      links,
      playlistName,
      startIndex,
      false,
    );
  };

  const handlePauseCascade = async (
    playlistId: string,
    links: { id: string; url: string; filename?: string }[],
  ) => {
    const c = cascadeState[playlistId];
    if (!c || !c.downloading) return;
    const current = links[c.index];
    if (!current) return;
    try {
      await invoke("integrated_pause_download", {
        id: String(c.currentId || `${Date.now()}_${c.index}`),
        url: current.url,
        progressKey: progressKeyForLink(playlistId, current.id),
      });
      updateCascadeForPlaylist(playlistId, (prev) => ({
        ...(prev || { index: 0, downloading: false, paused: true }),
        paused: true,
        downloading: false,
      }));
    } catch {
      toast.error("Erro ao pausar download da playlist");
    }
  };

  const handleResumeCascade = async (
    playlistId: string,
    links: { id: string; url: string; filename?: string }[],
    playlistName: string,
  ) => {
    const c = cascadeState[playlistId];
    const pausedIndex = links.findIndex(
      (_, idx) => linkProgress[linkKey(playlistId, idx)]?.status === "pausado",
    );
    const resumeIndex = c?.paused ? c.index : pausedIndex;
    if (resumeIndex < 0 || resumeIndex >= links.length) return;

    resumeGuardRef.current[linkKey(playlistId, resumeIndex)] =
      Date.now() + 30000;
    setLinkProgress((prev) => ({
      ...prev,
      [linkKey(playlistId, resumeIndex)]: {
        status: "baixando",
        progress: prev[linkKey(playlistId, resumeIndex)]?.progress || 0,
        total: prev[linkKey(playlistId, resumeIndex)]?.total || 0,
      },
    }));

    updateCascadeForPlaylist(playlistId, (prev) => ({
      ...(prev || { index: resumeIndex, downloading: false, paused: false }),
      index: resumeIndex,
      downloading: true,
      paused: false,
    }));
    await processCascadeFrom(
      playlistId,
      links,
      playlistName,
      resumeIndex,
      true,
    );
  };

  const handleLogout = (): void => {
    localStorage.removeItem("loginToken");
    localStorage.removeItem("lastUser");
    window.dispatchEvent(
      new StorageEvent("storage", {
        key: "loginToken",
      }),
    );
    navigate("/login", { replace: true });
  };
  const handleGoHome = (): void => {
    navigate("/");
  };

  // Abrir modal para criar nova playlist
  const openCreateModal = (): void => {
    setModalTitle("");
    setModalLinks([{ url: "", filename: "" }]);
    setEditId(null);
    setShowModal(true);
  };

  // Abrir modal para editar playlist existente
  const openEditModal = (playlist: Playlist): void => {
    setModalTitle(playlist.name);
    setModalLinks(
      playlist.links.length
        ? playlist.links.map((l) => ({
            url: l.url,
            filename: l.filename || "",
          }))
        : [{ url: "", filename: "" }],
    );
    setEditId(playlist.id);
    setShowModal(true);
  };

  // Confirmar exclusão de playlist
  const handleDeletePlaylist = async (): Promise<void> => {
    if (!deleteConfirmId || !username) return;
    try {
      const newPlaylists = playlists.filter((pl) => pl.id !== deleteConfirmId);
      await replacePanelPlaylists(username, newPlaylists);
      setPlaylists(newPlaylists);
      setDeleteConfirmId(null);
      toast.success("Playlist deletada com sucesso!", {
        position: "top-center",
        autoClose: 2000,
        hideProgressBar: true,
        closeOnClick: true,
        pauseOnHover: false,
        draggable: false,
      });
    } catch (error) {
      toast.error(`Erro ao deletar playlist: ${error}`);
    }
  };

  // Adiciona/remover campos de link na modal
  const handleAddLinkField = (): void =>
    setModalLinks((links) => [...links, { url: "", filename: "" }]);
  const handleRemoveLinkField = (idx: number): void =>
    setModalLinks((links) =>
      links.length > 1 ? links.filter((_, i) => i !== idx) : links,
    );
  const handleChangeLinkUrlField = (idx: number, value: string): void =>
    setModalLinks((links) =>
      links.map((l, i) => (i === idx ? { ...l, url: value } : l)),
    );
  const handleChangeLinkFilenameField = (idx: number, value: string): void =>
    setModalLinks((links) =>
      links.map((l, i) => (i === idx ? { ...l, filename: value } : l)),
    );

  // Salvar playlist (criar ou editar)
  const handleSavePlaylist = async (
    e: React.FormEvent<HTMLFormElement>,
  ): Promise<void> => {
    e.preventDefault();
    const title = modalTitle.trim();
    const links = modalLinks
      .map((l) => ({ url: l.url.trim(), filename: l.filename.trim() }))
      .filter((l) => Boolean(l.url));
    if (!title) return;
    let newPlaylists: Playlist[];
    if (editId) {
      newPlaylists = playlists.map((pl) =>
        pl.id === editId
          ? {
              ...pl,
              name: title,
              links: links.map((link) => ({
                id: Date.now().toString() + Math.random(),
                url: link.url,
                filename: link.filename,
              })),
            }
          : pl,
      );
    } else {
      newPlaylists = [
        {
          id: Date.now().toString(),
          name: title,
          links: links.map((link) => ({
            id: Date.now().toString() + Math.random(),
            url: link.url,
            filename: link.filename,
          })),
        },
        ...playlists,
      ];
    }
    if (username) {
      try {
        await replacePanelPlaylists(username, newPlaylists);
      } catch (error) {
        toast.error(`Erro ao salvar playlist: ${error}`);
        return;
      }
    }
    setPlaylists(newPlaylists);
    setShowModal(false);
    toast.success("Playlist salva com sucesso!", {
      position: "top-center",
      autoClose: 2000,
      hideProgressBar: true,
      closeOnClick: true,
      pauseOnHover: false,
      draggable: false,
    });
  };
  const handleOpenFolder = (playlistName: string) => {
    openDownloadFolder(playlistName)
      .then(() => {
        // Mensagem de sucesso já é exibida pelo backend, não exibir nada aqui
      })
      .catch(() => {
        // Mensagem de erro já é exibida pelo backend, não exibir nada aqui
      });
  };

  return (
    <Container>
      <TopBarFixed>
        <NavButton onClick={handleGoHome}>Home</NavButton>
        <NavButton onClick={handleLogout}>Logout</NavButton>
      </TopBarFixed>
      <WelcomeWrapper>
        <Card>
          <Header>
            <WelcomeHeaderRow>
              <IconCircle>
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  className="h-6 w-6"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="white"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M16 21v-2a4 4 0 00-8 0v2M12 11a4 4 0 100-8 4 4 0 000 8z"
                  />
                </svg>
              </IconCircle>
              <Username>Bem-vindo, {username}</Username>
            </WelcomeHeaderRow>
          </Header>
        </Card>
      </WelcomeWrapper>
      <CreatePlaylistWrapper>
        <CreateButton onClick={openCreateModal}>Criar Playlist</CreateButton>
      </CreatePlaylistWrapper>
      <PlaylistsWrapper>
        {playlists.length === 0 && (
          <NoPlaylists>Nenhuma playlist criada ainda.</NoPlaylists>
        )}
        {playlists.map((pl) => {
          const cascade = cascadeState[pl.id];
          return (
            <PlaylistCard key={pl.id}>
              <PlaylistCardHeader>
                <PlaylistCardTitle>{pl.name}</PlaylistCardTitle>
                <div style={{ display: "flex", gap: "8px" }}>
                  <EditButton onClick={() => openEditModal(pl)}>
                    Editar
                  </EditButton>
                  <DeleteButton onClick={() => setDeleteConfirmId(pl.id)}>
                    Deletar
                  </DeleteButton>
                </div>
              </PlaylistCardHeader>
              <PlaylistLinksLabel>Links:</PlaylistLinksLabel>
              <PlaylistLinksList>
                {pl.links.length === 0 && <NoLinkItem>Nenhum link</NoLinkItem>}
                {pl.links.map((link, idx) =>
                  (() => {
                    const p = linkProgress[linkKey(pl.id, idx)];
                    const total = p?.total || 0;
                    const progress = p?.progress || 0;
                    const percent =
                      total > 0 ? Math.min((progress / total) * 100, 100) : 0;
                    const status = p?.status || "pendente";
                    const hasCustomName = Boolean(
                      link.filename && link.filename.trim(),
                    );
                    const effectiveName = hasCustomName
                      ? (link.filename || "").trim()
                      : `video_${idx + 1}.mp4`;
                    return (
                      <PlaylistLinkItem
                        key={link.id}
                        style={
                          cascade &&
                          cascade.downloading &&
                          cascade.index === idx
                            ? { borderColor: "#6671d4" }
                            : {}
                        }
                      >
                        <PlaylistLinkIndex>{idx + 1}</PlaylistLinkIndex>
                        <PlaylistLinkContent>
                          <PlaylistUrlText title={link.url}>
                            {link.url}
                          </PlaylistUrlText>
                          <PlaylistFilenameText>
                            Nome do arquivo: {effectiveName}
                            {!hasCustomName ? " (padrão)" : ""}
                          </PlaylistFilenameText>
                          <ProgressTrack>
                            <ProgressFill $percent={percent} $status={status} />
                          </ProgressTrack>
                          <ProgressMeta>
                            {status} - {Math.round(percent)}%
                          </ProgressMeta>
                        </PlaylistLinkContent>
                        {cascade &&
                          cascade.downloading &&
                          cascade.index === idx && (
                            <DownloadingBadge>(baixando...)</DownloadingBadge>
                          )}
                      </PlaylistLinkItem>
                    );
                  })(),
                )}
              </PlaylistLinksList>
              <PlaylistActionsRow>
                <EditButton
                  onClick={() =>
                    handleCascadeDownload(pl.id, pl.links, pl.name)
                  }
                  disabled={
                    pl.links.length === 0 ||
                    (cascade && (cascade.downloading || cascade.paused))
                  }
                  style={{ flex: "1 1 auto", minWidth: "140px" }}
                >
                  <FiDownload size={15} />
                  Baixar em cascata
                </EditButton>
                <EditButton
                  onClick={() => handlePauseCascade(pl.id, pl.links)}
                  disabled={!cascade || !cascade.downloading}
                  style={{
                    background:
                      "linear-gradient(135deg, #ff6b7a 0%, #ff4f67 100%)",
                    color: "#fff",
                    flex: "1 1 auto",
                    minWidth: "120px",
                  }}
                >
                  <FiPause size={15} />
                  Pausar
                </EditButton>
                <EditButton
                  onClick={() => handleResumeCascade(pl.id, pl.links, pl.name)}
                  disabled={!cascade || !cascade.paused}
                  style={{
                    background:
                      "linear-gradient(135deg, #33d08f 0%, #1ebf7d 100%)",
                    color: "#fff",
                    flex: "1 1 auto",
                    minWidth: "120px",
                  }}
                >
                  <FiPlay size={15} />
                  Continuar
                </EditButton>
                <EditButton
                  onClick={() => handleOpenFolder(pl.name)}
                  style={{
                    background:
                      "linear-gradient(135deg, #4d63d8 0%, #3f54c7 100%)",
                    color: "#fff",
                    flex: "1 1 auto",
                    minWidth: "120px",
                  }}
                >
                  <FiFolder size={15} />
                  Abrir pasta
                </EditButton>
              </PlaylistActionsRow>
              {cascade && (cascade.downloading || cascade.paused) && (
                <CascadeIndicator style={{ marginTop: "12px" }}>
                  {cascade.paused ? "Pausado em" : "Baixando"} vídeo{" "}
                  {Math.min(cascade.index + 1, pl.links.length)} de{" "}
                  {pl.links.length}
                </CascadeIndicator>
              )}
            </PlaylistCard>
          );
        })}
      </PlaylistsWrapper>
      <ToastContainer
        position="top-right"
        autoClose={3000}
        hideProgressBar={false}
        newestOnTop
        closeOnClick
        pauseOnFocusLoss
        draggable
        pauseOnHover
        theme="light"
      />
      {showModal && (
        <ModalOverlay>
          <ModalForm onSubmit={handleSavePlaylist}>
            <ModalTitle>
              {editId ? "Editar Playlist" : "Nova Playlist"}
            </ModalTitle>
            <ModalInput
              type="text"
              placeholder="Título da playlist"
              value={modalTitle}
              onChange={(e) => setModalTitle(e.target.value)}
              required
            />
            <ModalLinksLabel>URLs dos vídeos</ModalLinksLabel>
            {modalLinks.map((item, idx) => (
              <ModalLinkFieldWrapper key={idx}>
                <ModalLinkInputsColumn>
                  <ModalLinkInput
                    type="text"
                    placeholder={`URL #${idx + 1}`}
                    value={item.url}
                    onChange={(e) =>
                      handleChangeLinkUrlField(idx, e.target.value)
                    }
                    required={idx === 0}
                  />
                  <ModalLinkInput
                    type="text"
                    placeholder={`Nome #${idx + 1}`}
                    value={item.filename}
                    onChange={(e) =>
                      handleChangeLinkFilenameField(idx, e.target.value)
                    }
                  />
                </ModalLinkInputsColumn>
                <ModalLinkButtonsColumn>
                  {modalLinks.length > 1 && (
                    <ModalButton
                      type="button"
                      onClick={() => handleRemoveLinkField(idx)}
                    >
                      -
                    </ModalButton>
                  )}
                  {idx === modalLinks.length - 1 && (
                    <ModalButtonAdd type="button" onClick={handleAddLinkField}>
                      +
                    </ModalButtonAdd>
                  )}
                </ModalLinkButtonsColumn>
              </ModalLinkFieldWrapper>
            ))}
            <ModalActions>
              <SubmitButton type="submit">
                {editId ? "Salvar" : "Criar"}
              </SubmitButton>
              <CancelButton type="button" onClick={() => setShowModal(false)}>
                Cancelar
              </CancelButton>
            </ModalActions>
          </ModalForm>
        </ModalOverlay>
      )}
      {deleteConfirmId && (
        <ModalOverlay onClick={() => setDeleteConfirmId(null)}>
          <ConfirmModal onClick={(e) => e.stopPropagation()}>
            <ConfirmTitle>Confirmar exclusão</ConfirmTitle>
            <ConfirmText>
              Tem certeza de que deseja deletar esta playlist? Esta ação não
              pode ser desfeita.
            </ConfirmText>
            <ConfirmActions>
              <ConfirmDeleteButton onClick={() => void handleDeletePlaylist()}>
                Deletar
              </ConfirmDeleteButton>
              <ConfirmCancelButton onClick={() => setDeleteConfirmId(null)}>
                Cancelar
              </ConfirmCancelButton>
            </ConfirmActions>
          </ConfirmModal>
        </ModalOverlay>
      )}
    </Container>
  );
};

export default UserPanel;
