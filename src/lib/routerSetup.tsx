import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import LoginScreen from '../pages/LoginScreen/LoginScreen';
import RegisterScreen from '../pages/RegisterScreen/RegisterScreen';
import Home from '../pages/Home/Home';
import UserPanel from '../pages/UserPanel/UserPanel';
import PlaylistPanel from '../pages/PlaylistPanel/PlaylistPanel';

import type { User } from '@/types/user';

export default function RouterSetup({ user, setUser }: { user: User | null, setUser: (u: User | null) => void }) {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/login" element={<LoginScreen onLogin={(username) => setUser({ username })} />} />
        <Route path="/register" element={<RegisterScreen onRegister={() => {}} />} />
        <Route path="/" element={user ? <Home /> : <Navigate to="/login" />} />
        <Route path="/user" element={user ? <UserPanel /> : <Navigate to="/login" />} />
        <Route path="/playlists" element={user ? <PlaylistPanel playlists={[]} onDownload={() => {}} /> : <Navigate to="/login" />} />
      </Routes>
    </BrowserRouter>
  );
}
