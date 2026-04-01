import styled from "styled-components";

export const TopBarFixed = styled.div`
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  z-index: 100;
  display: flex;
  justify-content: space-between;
  background: transparent;
  border-bottom: none;
  box-shadow: none;
  padding: 24px 32px 0 32px;
  box-sizing: border-box;
`;

export const WelcomeHeaderRow = styled.div`
  display: flex;
  align-items: center;
  gap: 12px;
`;

export const Container = styled.div`
  min-height: 100vh;
  width: 100vw;
  display: flex;
  flex-direction: column;
  align-items: stretch;
  background: linear-gradient(135deg, #23243a 0%, #3a3b5a 60%, #18192a 100%);
`;

export const WelcomeWrapper = styled.div`
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-top: 120px;
  margin-bottom: 24px;
`;

export const CreatePlaylistWrapper = styled.div`
  width: 100%;
  display: flex;
  justify-content: center;
  margin-bottom: 32px;
`;

export const PlaylistsWrapper = styled.div`
  width: 100%;
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(340px, 1fr));
  gap: 32px;
  justify-items: center;
  align-items: flex-start;
  padding: 0 32px 32px 32px;
  box-sizing: border-box;
`;

export const PlaylistCard = styled.div`
  background: linear-gradient(160deg, #252b66 0%, #1b1f4e 55%, #191f44 100%);
  border: 1px solid #4f58a0;
  border-radius: 18px;
  box-shadow: 0 14px 34px #0a0d285e;
  padding: 22px;
  width: 100%;
  max-width: 900px;
  min-width: 0;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  gap: 14px;
`;

export const PlaylistCardHeader = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
`;

export const PlaylistCardTitle = styled.div`
  font-weight: 700;
  font-size: 1.85rem;
  color: #f7f8ff;
  letter-spacing: 0.2px;
`;

export const PlaylistLinksLabel = styled.div`
  color: #c3c9ff;
  font-size: 0.98rem;
  font-weight: 600;
  margin-top: 6px;
  margin-bottom: 8px;
`;

export const PlaylistLinksList = styled.ul`
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  width: 100%;
  min-width: 0;
`;

export const PlaylistLinkItem = styled.li`
  background: linear-gradient(140deg, #353b79 0%, #30356d 100%);
  border: 1px solid #5662b1;
  color: #f8f9ff;
  border-radius: 12px;
  padding: 12px 14px 12px 12px;
  margin-bottom: 10px;
  display: flex;
  align-items: flex-start;
  font-size: 1.02rem;
  width: 100%;
  min-width: 0;
  box-sizing: border-box;
  overflow: hidden;
`;

export const PlaylistLinkIndex = styled.span`
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(145deg, #5d7bff 0%, #8c55ff 100%);
  color: #fff;
  font-weight: 800;
  font-size: 0.9rem;
  border-radius: 12px;
  min-width: 36px;
  height: 36px;
  padding: 0 10px;
  margin-right: 14px;
  margin-top: 2px;
  flex-shrink: 0;
  align-self: flex-start;
  border: 1px solid #9ca8ff;
  box-shadow: 0 8px 18px #596dff70;
`;

export const PlaylistLinkContent = styled.div`
  text-align: left;
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex: 1;
  min-width: 0;
  width: 100%;
`;

export const PlaylistUrlText = styled.span`
  color: #ffffff;
  font-weight: 500;
  line-height: 1.35;
  display: block;
  width: 100%;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
`;

export const PlaylistFilenameText = styled.span`
  color: #d0d4ff;
  font-size: 0.82rem;
  font-weight: 600;
`;

export const ProgressTrack = styled.div`
  width: 100%;
  height: 8px;
  border-radius: 999px;
  background: #171c44;
  border: 1px solid #2b356f;
  overflow: hidden;
`;

export const ProgressFill = styled.div<{ $percent: number; $status: string }>`
  width: ${({ $percent }) => `${$percent}%`};
  height: 100%;
  background: ${({ $status }) =>
    $status === "concluído"
      ? "linear-gradient(90deg, #21b76a 0%, #49cc87 100%)"
      : $status === "erro"
        ? "linear-gradient(90deg, #cc4458 0%, #e75f72 100%)"
        : $status === "pausado"
          ? "linear-gradient(90deg, #a07a2d 0%, #c89b3a 100%)"
          : "linear-gradient(90deg, #5b6dff 0%, #8c5eff 100%)"};
  transition: width 0.3s ease;
`;

export const ProgressMeta = styled.span`
  font-size: 0.78rem;
  color: #d8ddff;
  letter-spacing: 0.2px;
`;

export const DownloadingBadge = styled.span`
  margin-left: 8px;
  font-weight: 700;
  color: #ffffff;
  background: linear-gradient(135deg, #596dff 0%, #7e58ff 100%);
  border: 1px solid #8795ff;
  border-radius: 999px;
  padding: 4px 10px;
  font-size: 0.75rem;
  white-space: nowrap;
`;

export const PlaylistActionsRow = styled.div`
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-top: 8px;
  align-items: center;
`;

export const CascadeIndicator = styled.span`
  color: #d0d4ff;
  font-weight: 600;
  align-self: center;
  font-size: 0.9rem;
`;

export const NoPlaylists = styled.div`
  color: #aaa;
  font-size: 18px;
`;

export const EditButton = styled.button`
  background: linear-gradient(135deg, #4f6bff 0%, #5f4dff 100%);
  color: #fff;
  border: none;
  border-radius: 10px;
  padding: 8px 16px;
  font-size: 0.92rem;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  line-height: 1;
  cursor: pointer;
  font-weight: 700;
  box-shadow: 0 6px 18px #4f6bff33;
  transition:
    background 0.2s ease,
    transform 0.15s ease,
    opacity 0.2s ease,
    box-shadow 0.2s ease;
  &:hover {
    background: linear-gradient(135deg, #5f78ff 0%, #6f59ff 100%);
    transform: translateY(-1px);
    box-shadow: 0 8px 20px #4f6bff55;
  }
  &:disabled {
    opacity: 0.72;
    cursor: not-allowed;
    transform: none;
    box-shadow: none;
  }
`;

export const CreateButton = styled.button`
  background: #6c63ff;
  color: #fff;
  border: none;
  border-radius: 8px;
  font-size: 18px;
  padding: 10px 32px;
  cursor: pointer;
  font-weight: 700;
`;

export const SubmitButton = styled.button`
  background: linear-gradient(90deg, #6c63ff 60%, #a463ff 100%);
  color: #fff;
  border: none;
  border-radius: 10px;
  font-size: 1rem;
  font-weight: 700;
  padding: 10px 18px;
  min-width: 120px;
  box-shadow: 0 2px 8px #6c63ff33;
  cursor: pointer;
  transition:
    background 0.2s,
    box-shadow 0.2s,
    opacity 0.2s;
  &:hover {
    background: linear-gradient(90deg, #5a54e6 60%, #8a54e6 100%);
    box-shadow: 0 4px 16px #6c63ff55;
  }
  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
`;

export const NoLinkItem = styled.li`
  color: #aaa;
`;

export const CancelButton = styled.button`
  background: #363759;
  color: #fff;
  border: none;
  border-radius: 10px;
  font-size: 1rem;
  font-weight: 700;
  padding: 10px 18px;
  min-width: 120px;
  box-shadow: 0 2px 8px #23243a33;
  cursor: pointer;
  transition:
    background 0.2s,
    box-shadow 0.2s,
    opacity 0.2s;
  &:hover {
    background: #23243a;
    box-shadow: 0 4px 16px #23243a55;
  }
  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
`;

export const Card = styled.div`
  width: 100%;
  max-width: 520px;
  min-width: 320px;
  background: rgba(35, 36, 58, 0.95);
  box-shadow: 0 8px 32px #0004;
  border-radius: 24px;
  padding: 40px 32px;
  border: 1px solid #363759;
  backdrop-filter: blur(8px);
  display: flex;
  flex-direction: column;
  gap: 24px;
`;

export const Header = styled.header`
  display: flex;
  justify-content: space-between;
  align-items: center;
`;

export const IconCircle = styled.div`
  background: linear-gradient(135deg, #6c63ff 60%, #a463ff 100%);
  border-radius: 50%;
  padding: 12px;
  box-shadow: 0 4px 16px #6c63ff44;
  display: flex;
  align-items: center;
  justify-content: center;
`;

export const Username = styled.h2`
  font-size: 1.5rem;
  font-weight: 700;
  color: #fff;
`;

export const NavButton = styled.button`
  background: #6c63ff;
  color: #fff;
  font-weight: 700;
  font-size: 1rem;
  padding: 8px 24px;
  border-radius: 10px;
  border: none;
  cursor: pointer;
  transition:
    background 0.2s,
    color 0.2s,
    box-shadow 0.2s;
  margin-left: 12px;
  margin-right: 0;
  box-shadow: 0 2px 8px #6c63ff33;
  &:hover {
    background: #4b47c4;
    color: #fff;
    box-shadow: 0 4px 16px #6c63ff55;
  }
`;

export const ModalOverlay = styled.div`
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: #000a;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
`;

export const ModalForm = styled.form`
  background: #23234a;
  border-radius: 16px;
  padding: 40px 36px 32px 36px;
  width: min(980px, 92vw);
  min-width: 760px;
  max-width: 980px;
  box-shadow: 0 4px 32px #0008;
  display: flex;
  flex-direction: column;
  gap: 22px;

  @media (max-width: 820px) {
    min-width: 0;
    width: 96vw;
    padding: 24px 18px 20px 18px;
  }
`;

export const ModalTitle = styled.div`
  font-weight: 700;
  font-size: 22px;
  color: #fff;
  margin-bottom: 8px;
`;

export const ModalInput = styled.input`
  padding: 10px 14px;
  border-radius: 8px;
  border: none;
  outline: none;
  background: #363759;
  color: #fff;
  font-size: 17px;
  margin-bottom: 6px;
`;

export const ModalLinksLabel = styled.div`
  color: #b3b3ff;
  font-weight: 600;
  font-size: 15px;
  margin-bottom: 4px;
`;

export const ModalLinkFieldWrapper = styled.div`
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 8px;
  margin-bottom: 4px;

  @media (max-width: 820px) {
    grid-template-columns: 1fr;
  }
`;

export const ModalLinkInputsColumn = styled.div`
  display: grid;
  grid-template-columns: 1fr;
  gap: 8px;
`;

export const ModalLinkButtonsColumn = styled.div`
  display: flex;
  align-items: center;
  gap: 8px;

  @media (max-width: 820px) {
    justify-content: flex-end;
  }
`;

export const ModalLinkInput = styled.input`
  flex: 1;
  padding: 8px 12px;
  border-radius: 8px;
  border: none;
  outline: none;
  background: #363759;
  color: #fff;
  font-size: 16px;
`;

export const ModalButton = styled.button`
  background: #ff5e5e;
  color: #fff;
  border: none;
  border-radius: 8px;
  padding: 0 10px;
  cursor: pointer;
  font-weight: 700;
  font-size: 18px;
`;

export const ModalButtonAdd = styled.button`
  background: #6c63ff;
  color: #fff;
  border: none;
  border-radius: 8px;
  padding: 0 12px;
  cursor: pointer;
  font-weight: 700;
  font-size: 18px;
`;

export const ModalActions = styled.div`
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 10px;

  @media (max-width: 820px) {
    justify-content: stretch;
  }
`;
