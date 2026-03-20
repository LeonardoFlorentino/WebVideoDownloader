import { useState, useEffect } from "react";
import type { ReactNode } from "react";
import { loadDownloads, saveDownloads } from "../service/downloadsService";
import type { Download } from "@/types/download";

// Mover DownloadsContext para um arquivo separado para Fast Refresh
// src/context/DownloadsContextValue.ts
// export const DownloadsContext = createContext<DownloadsContextType | undefined>(undefined);
// Aqui mantemos apenas o provider
import { DownloadsContext } from "./DownloadsContextValue.ts";

export function DownloadsProvider({
  children,
  username,
}: {
  children: ReactNode;
  username: string;
}) {
  const [downloads, setDownloads] = useState<Download[]>(() =>
    loadDownloads(username),
  );

  // Atualiza downloads sempre que username mudar
  useEffect(() => {
    setDownloads(loadDownloads(username));
  }, [username]);

  // Salva downloads sempre que downloads mudar
  useEffect(() => {
    if (username) saveDownloads(username, downloads);
  }, [downloads, username]);

  return (
    <DownloadsContext.Provider
      value={{ downloads, setDownloads, username, setUsername: () => {} }}
    >
      {children}
    </DownloadsContext.Provider>
  );
}
