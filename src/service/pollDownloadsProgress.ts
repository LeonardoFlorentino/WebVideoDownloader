import { invoke } from "@tauri-apps/api/core";
import type { Download } from "@/types/download";

// Busca o progresso de todos os downloads ativos por polling
export async function pollDownloadsProgress(
  downloads: Download[],
): Promise<Download[]> {
  return Promise.all(
    downloads.map(async (d) => {
      try {
        const result = await invoke("get_progress_command", { url: d.url });
        if (
          result &&
          typeof result === "object" &&
          "ok" in result &&
          (result as any).ok &&
          "data" in result &&
          (result as any).data
        ) {
          const data = (result as any).data;
          // Log para depuração
          console.log("[pollDownloadsProgress] url:", d.url, "status recebido:", data.status, data);
          return {
            ...d,
            progress:
              typeof data.downloaded === "number"
                ? data.downloaded
                : 0,
            total:
              typeof data.total_size === "number"
                ? data.total_size
                : undefined,
            status: data.status || d.status,
          };
        }
      } catch (e) {
        console.warn("[pollDownloadsProgress] erro ao buscar progresso para", d.url, e);
      }
      return d;
    }),
  );
}
