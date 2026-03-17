export const TitleWrapper = styled.div`
  display: flex;
  flex-direction: column;
  align-items: center;
`;
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
  max-width: 420px;
  min-width: 320px;
  background: rgba(35,36,58,0.95);
  box-shadow: 0 8px 32px #0004;
  border-radius: 24px;
  padding: 40px 32px;
  border: 1px solid #363759;
  backdrop-filter: blur(8px);
  display: flex;
  flex-direction: column;
  gap: 32px;
`;
export const IconCircle = styled.div`
  background: linear-gradient(135deg, #6c63ff 60%, #a463ff 100%);
  border-radius: 50%;
  padding: 18px;
  margin-bottom: 12px;
  box-shadow: 0 4px 16px #6c63ff44;
  display: flex;
  align-items: center;
  justify-content: center;
`;
export const Title = styled.h2`
  font-size: 2.4rem;
  font-weight: 800;
  color: #fff;
  margin-bottom: 4px;
  text-align: center;
`;
export const Subtitle = styled.p`
  color: #b3b3ff;
  font-size: 1.1rem;
  text-align: center;
`;
export const Form = styled.form`
  display: flex;
  flex-direction: column;
  gap: 18px;
`;
export const Input = styled.input`
  padding: 14px 16px;
  border-radius: 12px;
  border: 1px solid #363759;
  background: #23243a;
  color: #fff;
  font-size: 1.1rem;
  margin-bottom: 2px;
  outline: none;
  transition: border 0.2s;
  &:focus {
    border-color: #6c63ff;
  }
`;
export const Button = styled.button`
  padding: 14px 0;
  border-radius: 12px;
  font-size: 1.15rem;
  font-weight: 700;
  color: #fff;
  background: linear-gradient(90deg, #6c63ff 60%, #a463ff 100%);
  border: none;
  box-shadow: 0 2px 8px #6c63ff33;
  cursor: pointer;
  margin-top: 4px;
  transition: background 0.2s, opacity 0.2s;
  &:hover {
    background: linear-gradient(90deg, #5a54e6 60%, #8a54e6 100%);
  }
  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
`;
export const ButtonAlt = styled(Button)`
  background: linear-gradient(90deg, #a463ff 60%, #6c63ff 100%);
  margin-top: 0;
`;
export const ErrorMsg = styled.div`
  color: #ff5e5e;
  font-size: 1rem;
  text-align: center;
  font-weight: 600;
  margin-top: 8px;
`;
export const SuccessMsg = styled.div`
  color: #43b581;
  font-size: 1rem;
  text-align: center;
  font-weight: 600;
  margin-top: 8px;
`;
