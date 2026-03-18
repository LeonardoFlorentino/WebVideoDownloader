import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import RouterSetup from './lib/routerSetup';

import type { User } from '@/types/user';


function App() {
  const [user, setUser] = useState<User | null>(null);

  useEffect(() => {
    const saved = localStorage.getItem('rememberedLogin');
    if (saved) {
      try {
        const { username, password } = JSON.parse(saved);
        if (username && password) {
          invoke('autenticar_usuario', { username, password })
            .then(() => {
              setUser({ username, links: [] });
            })
            .catch(() => {
              // Se falhar, não faz nada (usuário não autenticado)
            });
        }
      } catch {}
    }
  }, []);

  return <RouterSetup user={user} setUser={setUser} />;
}

export default App;
