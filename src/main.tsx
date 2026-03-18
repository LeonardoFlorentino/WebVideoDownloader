
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import './index.css';
import App from './App.tsx';
import { DownloadsProvider } from './context/DownloadsContext';

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <DownloadsProvider>
      <App />
    </DownloadsProvider>
  </StrictMode>,
)
