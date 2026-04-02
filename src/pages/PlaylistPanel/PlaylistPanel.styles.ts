import styled from "styled-components";

export const Panel = styled.div`
  display: flex;
  flex-direction: column;
  gap: 24px;
  padding: 20px;
`;
export const PlaylistCard = styled.div`
  display: flex;
  flex-direction: column;
  gap: 12px;
  background: #23243a;
  border-radius: 16px;
  padding: 18px 24px;
  box-shadow: 0 2px 8px #0002;
  border: 1px solid #363759;
`;
export const PlaylistHeader = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
`;
export const Info = styled.div`
  display: flex;
  flex-direction: column;
`;
export const Title = styled.span`
  font-weight: 700;
  color: #fff;
  font-size: 1.15rem;
`;
export const Status = styled.span<{ downloaded: boolean }>`
  color: ${(props) => (props.downloaded ? "#43b581" : "#b3b3ff")};
  font-size: 1rem;
  font-weight: 500;
`;
export const ButtonGroup = styled.div`
  display: flex;
  gap: 8px;
  align-items: center;
`;
export const Button = styled.button`
  background: #6c63ff;
  color: #fff;
  font-weight: 600;
  padding: 8px 16px;
  border-radius: 8px;
  border: none;
  cursor: pointer;
  transition: all 0.2s;
  font-size: 0.9rem;
  &:hover {
    background: #4b47c4;
  }
  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
`;
export const DeleteButton = styled(Button)`
  background: linear-gradient(135deg, #ff6b7a 0%, #ff4f67 100%);
  &:hover {
    background: linear-gradient(135deg, #e55a69 0%, #ee3e56 100%);
  }
`;
export const Empty = styled.div`
  color: #b3b3ff;
  text-align: center;
  font-size: 1.1rem;
  padding: 40px 20px;
`;

// Modal styles
export const ModalOverlay = styled.div`
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
`;

export const ModalContent = styled.div`
  background: #23243a;
  border-radius: 12px;
  padding: 24px;
  max-width: 400px;
  border: 1px solid #363759;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);
`;

export const ModalTitle = styled.h2`
  color: #fff;
  margin: 0 0 16px 0;
  font-size: 1.2rem;
`;

export const ModalText = styled.p`
  color: #b3b3ff;
  margin: 0 0 24px 0;
  line-height: 1.5;
`;

export const ModalActions = styled.div`
  display: flex;
  gap: 12px;
  justify-content: flex-end;
`;

export const ConfirmButton = styled(Button)`
  background: linear-gradient(135deg, #ff6b7a 0%, #ff4f67 100%);
  &:hover {
    background: linear-gradient(135deg, #e55a69 0%, #ee3e56 100%);
  }
`;

export const CancelButton = styled(Button)`
  background: #4a4a6a;
  &:hover {
    background: #5a5a7a;
  }
`;
