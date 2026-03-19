import { useState, useEffect } from "react";
import RouterSetup from "./lib/routerSetup";
import { DownloadsProvider } from "./context/DownloadsContext";
import type { User } from "@/types/user";

function isTokenValid(token: string | null) {
  if (!token) return false;
  try {
    const { expires } = JSON.parse(atob(token));
    return Date.now() < expires;
  } catch {
    return false;
  }
}

function getUsernameFromToken(token: string | null): string | null {
  if (!token) return null;
  try {
    const { username } = JSON.parse(atob(token));
    return username || null;
  } catch {
    return null;
  }
}

function App() {
  const [user, setUser] = useState<User | null>(null);
  console.log("[App] INÍCIO - user:", user);

  useEffect(() => {
    console.log("[App] Renderizou. user:", user);
  });

  // Atualiza user ao montar e quando loginToken mudar
  useEffect(() => {
    const updateUserFromToken = () => {
      const token = localStorage.getItem("loginToken");
      console.log("[App] updateUserFromToken token:", token);
      if (isTokenValid(token)) {
        const username = getUsernameFromToken(token);
        console.log("[App] Token válido, username:", username);
        if (username) {
          setUser((prev) => {
            console.log("[App] setUser chamado, username:", username);
            return { username };
          });
        } else {
          setUser(null);
        }
      } else {
        console.log("[App] Token inválido ou ausente");
        setUser(null);
      }
    };
    updateUserFromToken();
    // Escuta mudanças no localStorage (login/logout em outras abas)
    window.addEventListener("storage", (e) => {
      if (e.key === "loginToken") {
        updateUserFromToken();
      }
    });
    return () => {
      window.removeEventListener("storage", updateUserFromToken);
    };
  }, []);

  return (
    <DownloadsProvider username={user?.username || ""}>
      <RouterSetup user={user} setUser={setUser} />
    </DownloadsProvider>
  );
}

export default App;
