// Centralized API base URL resolution.
// In Tauri (desktop) we want to call the local server directly.
// In web/dev we can keep relative paths to use the SvelteKit proxy.

export const API_BASE: string =
  typeof window !== 'undefined' && '__TAURI__' in window
    ? 'http://127.0.0.1:7370'
    : '';

export function api(path: string): string {
  const p = path.startsWith('/') ? path : `/${path}`;
  return `${API_BASE}${p}`;
}
