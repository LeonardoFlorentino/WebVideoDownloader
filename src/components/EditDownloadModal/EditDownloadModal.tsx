import React, { useState } from "react";
import styled from "styled-components";

export interface EditDownloadModalProps {
  isOpen: boolean;
  initialFilename: string;
  initialUrl: string;
  onSave: (filename: string, url: string) => void;
  onCancel: () => void;
}

const Overlay = styled.div`
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
`;

const Modal = styled.div`
  background: #23284d;
  border-radius: 18px;
  padding: 48px 56px 40px 56px;
  min-width: 600px;
  min-height: 340px;
  max-width: 98vw;
  max-height: 98vh;
  box-shadow: 0 6px 32px #000a;
  display: flex;
  flex-direction: column;
  gap: 32px;
  align-items: center;
`;

const Title = styled.h2`
  color: #fff;
  font-size: 1.6rem;
  font-weight: 700;
  margin: 0 0 18px 0;
  letter-spacing: 0.5px;
  text-align: center;
`;

const Input = styled.input`
  padding: 14px 16px;
  border-radius: 8px;
  border: 1.5px solid #6c63ff;
  background: #23284d;
  color: #fff;
  width: 100%;
  margin-bottom: 18px;
  font-size: 1.08rem;
  outline: none;
  box-shadow: 0 2px 8px #0002;
  transition: border 0.2s;
  &:focus {
    border: 1.5px solid #554ee3;
    background: #262b50;
  }
`;

const Actions = styled.div`
  display: flex;
  gap: 18px;
  justify-content: center;
  margin-top: 10px;
`;

const Button = styled.button`
  padding: 12px 32px;
  border-radius: 8px;
  border: none;
  font-weight: 700;
  font-size: 1.08rem;
  cursor: pointer;
  background: linear-gradient(90deg, #6c63ff 60%, #554ee3 100%);
  color: #fff;
  box-shadow: 0 2px 8px #0002;
  transition:
    background 0.2s,
    transform 0.15s;
  &:hover {
    background: linear-gradient(90deg, #554ee3 60%, #6c63ff 100%);
    transform: translateY(-2px) scale(1.04);
  }
`;

const CancelButton = styled(Button)`
  background: linear-gradient(90deg, #e53935 60%, #b71c1c 100%);
  &:hover {
    background: linear-gradient(90deg, #b71c1c 60%, #e53935 100%);
  }
`;

export const EditDownloadModal: React.FC<EditDownloadModalProps> = ({
  isOpen,
  initialFilename,
  initialUrl,
  onSave,
  onCancel,
}) => {
  const [filename, setFilename] = useState(initialFilename);
  const [url, setUrl] = useState(initialUrl);

  if (!isOpen) return null;

  return (
    <Overlay>
      <Modal>
        <Title>Editar Download</Title>
        <Input
          type="text"
          placeholder="Nome do arquivo"
          value={filename}
          onChange={(e) => setFilename(e.target.value)}
        />
        <Input
          type="text"
          placeholder="URL do vídeo"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
        />
        <Actions>
          <Button
            type="button"
            onClick={() => onSave(filename.trim(), url.trim())}
            disabled={!filename.trim() || !url.trim()}
          >
            Salvar
          </Button>
          <CancelButton type="button" onClick={onCancel}>
            Cancelar
          </CancelButton>
        </Actions>
      </Modal>
    </Overlay>
  );
};
