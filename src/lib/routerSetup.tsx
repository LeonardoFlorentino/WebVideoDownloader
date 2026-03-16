import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import LoginScreen from '../pages/LoginScreen/LoginScreen';
import RegisterScreen from '../pages/RegisterScreen/RegisterScreen';
import Home from '../pages/Home/Home';
import UserPanel from '../pages/UserPanel/UserPanel';
import PlaylistPanel from '../pages/PlaylistPanel/PlaylistPanel';

interface User {
  username: string;
  links: Array<{ title: string; url: string; downloaded: boolean }>;
}

export default function RouterSetup({ user, setUser }: { user: User | null, setUser: (u: User | null) => void }) {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/login" element={<LoginScreen onLogin={(username) => setUser({ username, links: [] })} />} />
        <Route path="/register" element={<RegisterScreen onRegister={() => {}} />} />
        <Route path="/" element={user ? <Home links={user.links} onDownloadAll={() => {}} /> : <Navigate to="/login" />} />
        <Route path="/user" element={user ? <UserPanel username={user.username} /> : <Navigate to="/login" />} />
        <Route path="/playlists" element={user ? <PlaylistPanel playlists={[]} onDownload={() => {}} /> : <Navigate to="/login" />} />
      </Routes>
    </BrowserRouter>
  );
}
