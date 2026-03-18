import { createContext } from 'react';
import type { Download } from '@/types/download';

export interface DownloadsContextType {
  downloads: Download[];
  setDownloads: React.Dispatch<React.SetStateAction<Download[]>>;
  username: string;
  setUsername: React.Dispatch<React.SetStateAction<string>>;
}

export const DownloadsContext = createContext<DownloadsContextType | undefined>(undefined);
