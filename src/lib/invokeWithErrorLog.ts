import { invoke } from "@tauri-apps/api/core";

export async function invokeWithErrorLog<T = any>(
  cmd: string,
  args?: any,
): Promise<T | undefined> {
  const result = await invoke<any>(cmd, args);
  if (result && typeof result === "object" && "ok" in result) {
    if (!result.ok && result.error) {
      console.error(`[TAURI ERROR] ${cmd}:`, result.error);
      // Aqui você pode disparar um toast ou outro alerta se quiser
      return undefined;
    }
    return result.data;
  }
  return result;
}
