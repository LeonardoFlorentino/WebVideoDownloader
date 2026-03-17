import styled from 'styled-components';

export const Container = styled.div`
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #23243a 0%, #3a3b5a 60%, #18192a 100%);
`;
export const Card = styled.div`
  width: 100%;
  max-width: 600px;
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
export const Button = styled.button`
  background: #ff5e5e;
  color: #fff;
  font-weight: 600;
  padding: 10px 24px;
  border-radius: 12px;
  border: none;
  cursor: pointer;
  transition: background 0.2s;
  &:hover {
    background: #e53e3e;
  }
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
export const Section = styled.section`
  margin-top: 12px;
`;
export const Title = styled.h3`
  font-size: 1.2rem;
  font-weight: 600;
  color: #b3b3ff;
  margin-bottom: 12px;
`;
