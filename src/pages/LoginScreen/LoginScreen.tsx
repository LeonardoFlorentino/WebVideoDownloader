import React, { useState, useEffect } from "react";
// Função simples para gerar um token com expiração (7 dias)
function generateToken(username: string) {
  const expires = Date.now() + 7 * 24 * 60 * 60 * 1000; // 7 dias
  return btoa(JSON.stringify({ username, expires }));
}
function isTokenValid(token: string | null) {
  if (!token) return false;
  try {
    const { expires } = JSON.parse(atob(token));
    return Date.now() < expires;
  } catch {
    return false;
  }
}
// Ícone simples de olho (SVG)
const EyeIcon = ({ open }: { open: boolean }) => (
  <svg
    width="22"
    height="22"
    viewBox="0 0 24 24"
    fill="none"
    stroke="#b3b3ff"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    {open ? (
      <>
        <ellipse cx="12" cy="12" rx="8" ry="5" />
        <circle cx="12" cy="12" r="2.5" />
      </>
    ) : (
      <>
        <ellipse cx="12" cy="12" rx="8" ry="5" />
        <path d="M2 2l20 20" stroke="#b3b3ff" />
      </>
    )}
  </svg>
);
import { useNavigate } from "react-router-dom";
import {
  Container,
  Card,
  IconCircle,
  Title,
  Subtitle,
  Form,
  Input,
  Button,
  ButtonAlt,
  ErrorMsg,
  TitleWrapper,
  PasswordFieldWrapper,
  ShowPasswordButton,
  RememberLabel,
  RememberCheckbox,
} from "./LoginScreen.styles";
import { invoke } from "@tauri-apps/api/core";

export default function LoginScreen({
  onLogin,
}: {
  onLogin: (token: string) => void;
}) {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [remember, setRemember] = useState(false);
  const [showPassword, setShowPassword] = useState(false);
  const [loggedIn, setLoggedIn] = useState(false);
  const navigate = useNavigate();

  // Load remembered credentials or token
  useEffect(() => {
    const saved = localStorage.getItem("rememberedLogin");
    if (saved) {
      try {
        const { username, password } = JSON.parse(saved);
        setUsername(username || "");
        setPassword(password || "");
        setRemember(true);
      } catch {
        // Ignora erro ao ler rememberedLogin
      }
    }
    // Se já existe token válido, loga automaticamente (apenas se não estiver logado)
    if (!loggedIn) {
      const token = localStorage.getItem("loginToken");
      if (isTokenValid(token)) {
        onLogin(token!);
        setLoggedIn(true);
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    try {
      await invoke("autenticar_usuario_tauri", {
        username,
        password,
      });
      // Se não houve exceção, o login foi bem-sucedido (result == null)
      const token: string = generateToken(username);
      if (remember) {
        localStorage.setItem(
          "rememberedLogin",
          JSON.stringify({ username, password }),
        );
      } else {
        localStorage.removeItem("rememberedLogin");
      }
      localStorage.setItem("loginToken", token);
      localStorage.setItem("lastUser", username);
      onLogin(token);
      setLoggedIn(true);
    } catch (err) {
      console.error("Login: erro no invoke", err);
      setError((err as Error)?.toString() || "Usuário ou senha inválidos");
    }
    setLoading(false);
  };

  // Redireciona após login usando useEffect para evitar erro de atualização de estado durante o render
  useEffect(() => {
    if (loggedIn) {
      navigate("/");
    }
  }, [loggedIn, navigate]);
  return (
    <Container>
      <Card>
        <TitleWrapper>
          <IconCircle>
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-10 w-10"
              fill="none"
              viewBox="0 0 24 24"
              stroke="white"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M16 21v-2a4 4 0 00-8 0v2M12 11a4 4 0 100-8 4 4 0 000 8z"
              />
            </svg>
          </IconCircle>
          <Title>Login</Title>
          <Subtitle>Acesse sua conta para continuar</Subtitle>
        </TitleWrapper>
        <Form onSubmit={handleLogin}>
          <Input
            type="text"
            placeholder="Usuário"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            required
            autoFocus
          />
          <PasswordFieldWrapper>
            <Input
              type={showPassword ? "text" : "password"}
              placeholder="Senha"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
              style={{ paddingRight: 38 }}
            />
            <ShowPasswordButton
              type="button"
              aria-label={showPassword ? "Ocultar senha" : "Mostrar senha"}
              onClick={() => setShowPassword((v) => !v)}
            >
              <EyeIcon open={showPassword} />
            </ShowPasswordButton>
          </PasswordFieldWrapper>
          <RememberLabel>
            <RememberCheckbox
              type="checkbox"
              checked={remember}
              onChange={(e) => setRemember(e.target.checked)}
            />
            Salvar senha
          </RememberLabel>
          <Button type="submit" disabled={loading}>
            {loading ? "Entrando..." : "Entrar"}
          </Button>
          <ButtonAlt type="button" onClick={() => navigate("/register")}>
            Cadastrar
          </ButtonAlt>
          {error && <ErrorMsg>{error}</ErrorMsg>}
        </Form>
      </Card>
    </Container>
  );
}
