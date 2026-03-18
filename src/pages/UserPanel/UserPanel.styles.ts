import styled from 'styled-components';

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
  background: #23234a;
  border-radius: 16px;
  box-shadow: 0 2px 12px #0002;
  padding: 24px;
  width: 100%;
  max-width: 900px;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
`;

export const PlaylistCardHeader = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
`;

export const PlaylistCardTitle = styled.div`
  font-weight: 700;
  font-size: 20px;
  color: #fff;
`;

export const PlaylistLinksLabel = styled.div`
  color: #b3b3ff;
  font-size: 15px;
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
`;

export const PlaylistLinkItem = styled.li`
  background: #363759;
  color: #fff;
  border-radius: 8px;
  padding: 6px 10px 6px 0;
  margin-bottom: 6px;
  word-break: break-all;
  display: flex;
  align-items: center;
  font-size: 1.05rem;
  width: 100%;
`;

export const PlaylistLinkIndex = styled.span`
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(90deg, #6c63ff 60%, #a463ff 100%);
  color: #fff;
  font-weight: 800;
  font-size: 1rem;
  border-radius: 6px;
  width: 32px;
  height: 32px;
  margin-right: 14px;
  box-shadow: 0 2px 8px #6c63ff22;
`;

export const NoPlaylists = styled.div`
  color: #aaa;
  font-size: 18px;
`;

export const EditButton = styled.button`
  background: #6c63ff;
  color: #fff;
  border: none;
  border-radius: 8px;
  padding: 6px 18px;
  font-size: 15px;
  cursor: pointer;
  font-weight: 700;
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
  border-radius: 12px;
  font-size: 1.15rem;
  font-weight: 800;
  padding: 14px 0;
  min-width: 160px;
  flex: 1;
  box-shadow: 0 2px 8px #6c63ff33;
  cursor: pointer;
  transition: background 0.2s, box-shadow 0.2s, opacity 0.2s;
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
  border-radius: 12px;
  font-size: 1.15rem;
  font-weight: 800;
  padding: 14px 0;
  min-width: 160px;
  flex: 1;
  box-shadow: 0 2px 8px #23243a33;
  cursor: pointer;
  transition: background 0.2s, box-shadow 0.2s, opacity 0.2s;
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
  background: rgba(35,36,58,0.95);
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
  transition: background 0.2s, color 0.2s, box-shadow 0.2s;
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
  min-width: 480px;
  max-width: 700px;
  box-shadow: 0 4px 32px #0008;
  display: flex;
  flex-direction: column;
  gap: 22px;
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
  display: flex;
  gap: 8px;
  margin-bottom: 4px;
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
  gap: 12px;
  margin-top: 10px;
`;