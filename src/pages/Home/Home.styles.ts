export const EmptyMessage = styled.div`
  color: #b3b3ff;
  text-align: center;
  margin-top: 32px;
`;

export const UrlText = styled.div`
  color: #6c63ff;
  font-size: 13px;
  word-break: break-all;
`;

export const DownloadButton = styled.button<{ disabled?: boolean }>`
  background: #6c63ff;
  color: #fff;
  border: none;
  border-radius: 8px;
  padding: 6px 14px;
  cursor: ${({ disabled }) => (disabled ? 'not-allowed' : 'pointer')};
  margin-right: 8px;
  transition: background 0.2s;
  &:hover {
    background: #4b47c4;
  }
  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
`;
export const TopBar = styled.div`
  width: 100%;
  max-width: 100vw;
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 24px 40px 0 40px;
  box-sizing: border-box;
  margin-bottom: 0;
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
import styled from 'styled-components';

export const Container = styled.div`
  min-height: 100vh;
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-start;
  background: linear-gradient(135deg, #18192a 0%, #23243a 60%, #23243a 100%);
  padding: 0;
  overflow-x: hidden;
  box-sizing: border-box;
`;
export const Title = styled.h2`
  font-size: 2.1rem;
  font-weight: 800;
  color: #b3b3ff;
  margin-top: 48px;
  margin-bottom: 18px;
  text-align: center;
`;

export const UrlForm = styled.form`
  display: flex;
  gap: 12px;
  margin-bottom: 8px;
  width: 100%;
  max-width: 600px;
`;
export const UrlInput = styled.input`
  flex: 1;
  padding: 14px 16px;
  border-radius: 12px;
  border: 1.5px solid #363759;
  background: #23243a;
  color: #fff;
  font-size: 1.1rem;
  outline: none;
  transition: border 0.2s;
  &:focus {
    border-color: #6c63ff;
  }
`;
export const AddButton = styled.button`
  background: #6c63ff;
  color: #fff;
  font-weight: 700;
  font-size: 1.1rem;
  padding: 0 24px;
  border-radius: 12px;
  border: none;
  cursor: pointer;
  transition: background 0.2s;
  &:hover {
    background: #4b47c4;
  }
  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
`;
export const DownloadAllButton = styled.button`
  display: block;
  margin: 16px auto 12px auto;
  background: #43b581;
  color: #fff;
  font-weight: 800;
  font-size: 1.15rem;
  padding: 12px 36px;
  border-radius: 24px;
  border: none;
  cursor: pointer;
  box-shadow: 0 2px 8px #43b58133;
  letter-spacing: 0.5px;
  transition: background 0.2s, box-shadow 0.2s, opacity 0.2s;
  width: auto;
  min-width: 180px;
  text-align: center;
  &:hover {
    background: #2e8c5a;
    box-shadow: 0 4px 16px #43b58155;
  }
  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
`;

export const DownloadList = styled.ul`
  display: flex;
  flex-direction: column;
  gap: 18px;
  margin-top: 12px;
  width: 100%;
  max-width: 600px;
`;
export const DownloadItem = styled.li`
  display: flex;
  align-items: center;
  gap: 18px;
  background: #23243a;
  border: 1px solid #363759;
  border-radius: 12px;
  padding: 16px 18px;
  box-shadow: 0 2px 8px #6c63ff11;
`;
export const FileInfo = styled.div`
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  min-width: 120px;
  margin-right: 8px;
`;
export const FileName = styled.div`
  font-weight: 700;
  color: #fff;
  font-size: 1.08rem;
`;
export const FileExt = styled.div`
  font-size: 0.95rem;
  color: #b3b3ff;
  font-weight: 600;
`;
export const ProgressBar = styled.div`
  margin-top: 8px;
`;
export const ProgressTrack = styled.div`
  width: 100%;
  height: 8px;
  background: #363759;
  border-radius: 8px;
  overflow: hidden;
`;
export const ProgressFill = styled.div<{ status: string }>`
  height: 100%;
  background: ${({ status }) =>
    status === 'concluído' ? '#43b581' :
    status === 'baixando' ? '#6c63ff' :
    status === 'erro' ? '#ff5e5e' : '#b3b3ff'};
  border-radius: 8px;
  transition: width 0.5s;
`;
export const Status = styled.div<{ status: string }>`
  min-width: 90px;
  text-align: center;
  font-weight: 700;
  color: ${({ status }) =>
    status === 'concluído' ? '#43b581' :
    status === 'baixando' ? '#6c63ff' :
    status === 'erro' ? '#ff5e5e' : '#b3b3ff'};
`;
export const DownloadActions = styled.div`
  display: flex;
  align-items: center;
`;
export const Button = styled.button`
  background: #6c63ff;
  color: #fff;
  font-weight: 600;
  padding: 10px 24px;
  border-radius: 12px;
  border: none;
  cursor: pointer;
  margin-bottom: 18px;
  transition: background 0.2s;
  &:hover {
    background: #4b47c4;
  }
  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
`;
export const List = styled.ul`
  display: flex;
  flex-direction: column;
  gap: 12px;
  background: transparent;
`;
export const Item = styled.li`
  display: flex;
  flex-direction: column;
  @media (min-width: 768px) {
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
  }
  background: #23243a;
  border: 1px solid #363759;
  border-radius: 12px;
  padding: 16px 18px;
  box-shadow: 0 2px 8px #6c63ff11;
`;
export const Info = styled.div`
  flex: 1;
  min-width: 0;
`;
export const VideoTitle = styled.div`
  font-weight: 600;
  color: #fff;
  word-break: break-all;
`;
export const VideoUrl = styled.div`
  font-size: 0.95rem;
  color: #b3b3ff;
  word-break: break-all;
`;
// Removido: Status antigo para evitar conflito de nome
export const StatusOld = styled.span<{ downloaded: boolean }>`
  color: ${props => props.downloaded ? '#43b581' : '#b3b3ff'};
  font-weight: 500;
  margin-top: 8px;
  @media (min-width: 768px) {
    margin-top: 0;
    margin-left: 24px;
  }
`;
