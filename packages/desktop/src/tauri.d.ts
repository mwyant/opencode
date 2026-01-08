declare module "@tauri-apps/api/tauri" {
  export function invoke<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T>;
}
