import styled from "styled-components";

export const CardTopRow = styled.div`
  display: flex;
  align-items: center;
  gap: 18px;
  width: 100%;
  justify-content: space-between;
  flex-wrap: nowrap;
  min-height: 38px;
`;

export const CardUrlAndStatus = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 18px;
  flex: 1;
  min-width: 0;
  height: 100%;
`;
export const CardContainer = styled.div`
  background: #23284d;
  border-radius: 10px;
  box-shadow: 0 2px 12px #0003;
  padding: 18px 20px 14px 20px;
  margin-bottom: 18px;
  display: flex;
  flex-direction: column;
  gap: 8px;
`;

export const CardFileInfo = styled.div`
  display: flex;
  align-items: center;
  gap: 10px;
  max-width: 220px;
  min-width: 0;
  flex-shrink: 1;
  overflow: hidden;
`;

export const CardFileName = styled.span`
  font-weight: 500;
  color: #fff;
  font-size: 0.95em;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 160px;
  display: inline-block;
`;

export const CardFileExt = styled.span`
  color: #a0a0c0;
  font-size: 0.95em;
`;

export const CardUrlText = styled.div`
  color: #b0b0e0;
  font-size: 1.08em;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1 1 0%;
  min-width: 0;
  max-width: 100vw;
  display: flex;
  align-items: center;
  height: 100%;
  margin-left: 16px;
`;

export const CardProgressBar = styled.div`
  width: 100%;
  height: 7px;
  background: transparent;
  border-radius: 4px;
  margin: 6px 0 0 0;
`;

export const CardProgressTrack = styled.div`
  width: 100%;
  height: 100%;
  background: #1a1b2e;
  border-radius: 4px;
`;

export const CardProgressFill = styled.div<{ $status: string }>`
  height: 100%;
  border-radius: 4px;
  background: ${({ $status }) =>
    $status === "concluído"
      ? "#4caf50"
      : $status === "baixando"
        ? "#6c63ff"
        : $status === "erro"
          ? "#e53935"
          : "#6c63ff"};
  width: ${({ style }) => style?.width || "0%"};
  transition: width 0.3s;
`;

export const CardStatus = styled.div<{ $status: string }>`
  font-size: 1em;
  font-weight: 600;
  color: ${({ $status }) =>
    $status === "concluído"
      ? "#4caf50"
      : $status === "baixando"
        ? "#6c63ff"
        : $status === "erro"
          ? "#e53935"
          : "#fff"};
  margin-left: 18px;
  white-space: nowrap;
`;

export const CardActions = styled.div`
  display: flex;
  gap: 8px;
  margin-top: 8px;
  align-items: center;
`;

export const OpenFolderButton = styled.button`
  background: #6c63ff;
  color: #fff;
  border: none;
  border-radius: 6px;
  padding: 6px 16px;
  font-weight: 500;
  cursor: pointer;
  box-shadow: 0 1px 4px 0 #0002;
  transition: background 0.2s;
  outline: none;
  &:hover {
    background: #554fd1;
  }
`;
