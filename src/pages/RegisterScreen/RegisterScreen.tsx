
import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Container, Card, IconCircle, Title, Subtitle, Form, Input, Button, ButtonAlt, ErrorMsg, SuccessMsg } from './RegisterScreen.styles';
import { invoke } from '@tauri-apps/api/core';


export default function RegisterScreen({ onRegister }: { onRegister: () => void }) {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [success, setSuccess] = useState(false);
  const navigate = useNavigate();

  const handleRegister = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    setSuccess(false);
    try {
      if (username.length < 3 || password.length < 3) {
        setError('Preencha todos os campos corretamente');
        setLoading(false);
        return;
      }
      await invoke('cadastrar_usuario', { username, password });
      setSuccess(true);
      setTimeout(() => {
        onRegister();
        navigate('/login');
      }, 1000);
    } catch (err: any) {
      setError(err?.toString() || 'Erro ao cadastrar usuário');
    }
    setLoading(false);
  };

  return (
    <Container>
      <Card>
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
          <IconCircle>
            <svg xmlns='http://www.w3.org/2000/svg' className='h-10 w-10' fill='none' viewBox='0 0 24 24' stroke='white'><path strokeLinecap='round' strokeLinejoin='round' strokeWidth={2} d='M16 21v-2a4 4 0 00-8 0v2M12 11a4 4 0 100-8 4 4 0 000 8z' /></svg>
          </IconCircle>
          <Title>Cadastro</Title>
          <Subtitle>Crie sua conta para acessar</Subtitle>
        </div>
        <Form onSubmit={handleRegister}>
          <Input
            type="text"
            placeholder="Usuário"
            value={username}
            onChange={e => setUsername(e.target.value)}
            required
            autoFocus
          />
          <Input
            type="password"
            placeholder="Senha"
            value={password}
            onChange={e => setPassword(e.target.value)}
            required
          />
          <Button type="submit" disabled={loading}>
            {loading ? 'Cadastrando...' : 'Cadastrar'}
          </Button>
          <ButtonAlt type="button" onClick={() => navigate('/login')}>
            Voltar para Login
          </ButtonAlt>
          {error && <ErrorMsg>{error}</ErrorMsg>}
          {success && <SuccessMsg>Cadastro realizado com sucesso!</SuccessMsg>}
        </Form>
      </Card>
    </Container>
  );
}
