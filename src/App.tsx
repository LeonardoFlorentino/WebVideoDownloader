import { useState } from 'react';
import RouterSetup from './lib/routerSetup';

interface User {
  username: string;
  links: Array<{ title: string; url: string; downloaded: boolean }>;
}

function App() {
  const [user, setUser] = useState<User | null>(null);
  return <RouterSetup user={user} setUser={setUser} />;
}

export default App;
