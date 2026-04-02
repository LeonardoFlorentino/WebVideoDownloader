import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import LoginScreen from "../pages/LoginScreen/LoginScreen";
import RegisterScreen from "../pages/RegisterScreen/RegisterScreen";
import Home from "../pages/Home/Home";
import UserPanel from "../pages/UserPanel/UserPanel";

import type { User } from "@/types/user";

export default function RouterSetup({
  user,
  setUser,
}: {
  user: User | null;
  setUser: (u: User | null) => void;
}) {
  return (
    <BrowserRouter>
      <Routes>
        <Route
          path="/login"
          element={
            <LoginScreen
              onLogin={(token) => {
                // Salva o token e define o usuário
                localStorage.setItem("loginToken", token);
                try {
                  const parsed = JSON.parse(atob(token));
                  if (parsed && typeof parsed.username === "string") {
                    setUser({ username: parsed.username });
                  } else {
                    setUser(null);
                  }
                } catch {
                  setUser(null);
                }
              }}
            />
          }
        />
        <Route
          path="/register"
          element={<RegisterScreen onRegister={() => {}} />}
        />
        <Route
          path="/"
          element={
            user ? <Home username={user.username} /> : <Navigate to="/login" />
          }
        />
        <Route
          path="/user"
          element={user ? <UserPanel /> : <Navigate to="/login" />}
        />
        <Route
          path="/playlists"
          element={user ? <UserPanel /> : <Navigate to="/login" />}
        />
      </Routes>
    </BrowserRouter>
  );
}
