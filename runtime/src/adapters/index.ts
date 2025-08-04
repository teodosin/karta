/**
 * Runtime data adapter placeholders and minimal types.
 * Implementations will load/resolve bundles from paths, URLs, or direct data.
 */
// TODO: Replace this local KartaBundle placeholder with import("@karta/core").KartaBundle
// after workspaces are installed and path exports are available.
export interface KartaBundlePlaceholder {
  version: number;
  exportedAt: string;
  metadata?: { minRuntimeVersion?: string; [k: string]: unknown };
  nodes: unknown[];
  contexts: Record<string, { viewNodes: Record<string, unknown> }>;
  edges: unknown[];
  assets?: Record<string, unknown>;
}

/** Source kinds supported by the runtime */
export type FilePathSource = { kind: "file"; path: string };
export type UrlSource = { kind: "url"; url: string; init?: RequestInit };
export type DirectDataSource = { kind: "data"; data: unknown };

/** Union used by KartaRuntime.loadBundle */
export type AdapterSource = FilePathSource | UrlSource | DirectDataSource;

/** Minimal adapter interface (concrete adapters will implement) */
export interface RuntimeDataAdapter {
  canHandle(source: AdapterSource): boolean;
  load(source: AdapterSource): Promise<KartaBundlePlaceholder>;
}

/** Simple adapters (stubs) */
export class DirectDataAdapter implements RuntimeDataAdapter {
  canHandle(source: AdapterSource): boolean {
    return source.kind === "data";
  }
  async load(source: AdapterSource): Promise<KartaBundlePlaceholder> {
    return (source as DirectDataSource).data as KartaBundlePlaceholder;
  }
}

export class UrlAdapter implements RuntimeDataAdapter {
  canHandle(source: AdapterSource): boolean {
    return source.kind === "url";
  }
  async load(_source: AdapterSource): Promise<KartaBundlePlaceholder> {
    // TODO: implement fetch and parse JSON
    throw new Error("UrlAdapter not implemented yet");
  }
}

export class FilePathAdapter implements RuntimeDataAdapter {
  canHandle(source: AdapterSource): boolean {
    return source.kind === "file";
  }
  async load(_source: AdapterSource): Promise<KartaBundlePlaceholder> {
    // TODO: implement file read for Node/web bundlers as applicable
    throw new Error("FilePathAdapter not implemented yet");
  }
}

/** Default adapter order (prefer direct data, then url, then file) */
export const defaultAdapters: ReadonlyArray<RuntimeDataAdapter> = [
  new DirectDataAdapter(),
  new UrlAdapter(),
  new FilePathAdapter(),
];