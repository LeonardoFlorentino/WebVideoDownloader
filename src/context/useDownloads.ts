import { useContext } from 'react';
import { DownloadsContext } from './DownloadsContextValue';

export function useDownloads() {
  const ctx = useContext(DownloadsContext);
  if (!ctx) throw new Error('useDownloads must be used within DownloadsProvider');
  return ctx;
}
