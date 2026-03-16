import styled from 'styled-components';

export const Panel = styled.div`
  display: flex;
  flex-direction: column;
  gap: 24px;
`;
export const PlaylistCard = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: #23243a;
  border-radius: 16px;
  padding: 18px 24px;
  box-shadow: 0 2px 8px #0002;
  border: 1px solid #363759;
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
  color: ${props => props.downloaded ? '#43b581' : '#b3b3ff'};
  font-size: 1rem;
  font-weight: 500;
`;
export const Button = styled.button`
  background: #6c63ff;
  color: #fff;
  font-weight: 600;
  padding: 10px 24px;
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
export const Empty = styled.div`
  color: #b3b3ff;
  text-align: center;
  font-size: 1.1rem;
`;
