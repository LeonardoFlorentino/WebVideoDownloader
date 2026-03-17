import { useState } from 'react';
import RouterSetup from './lib/routerSetup';

import type { User } from '@/types/user';

function App() {
  const [user, setUser] = useState<User | null>(null);
  return <RouterSetup user={user} setUser={setUser} />;
}

export default App;
