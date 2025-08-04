import { defaultAdapters } from "./adapters";
import type { AdapterSource, KartaBundlePlaceholder, RuntimeDataAdapter } from "./adapters";

/** Runtime initialization options (kept minimal; expand as needed) */
export interface KartaRuntimeOptions {
  allowUnsafeHtml?: boolean;
}

/** Basic event map for the runtime (expand later) */
export interface KartaRuntimeEvents {
  "bundle:loaded": { contextId: string | null };
  "context:changed": { contextId: string | null };
  "error": { message: string; cause?: unknown };
}

type EventKey = keyof KartaRuntimeEvents;
type Listener<K extends EventKey> = (payload: KartaRuntimeEvents[K]) => void;

export class KartaRuntime {
  private listeners: Map<EventKey, Set<Function>> = new Map();
  private currentContextId: string | null = null;

  constructor(private readonly opts: KartaRuntimeOptions = {}) {}

  async loadBundle(source: AdapterSource): Promise<void> {
    const adapter: RuntimeDataAdapter | undefined = defaultAdapters.find(a => a.canHandle(source));
    if (!adapter) {
      this.emit("error", { message: "No adapter available for provided source" });
      return;
    }

    let bundle: KartaBundlePlaceholder | null = null;
    try {
      bundle = await adapter.load(source);
    } catch (e) {
      this.emit("error", { message: "Failed to load bundle", cause: e });
      return;
    }

    if (!bundle) {
      this.emit("error", { message: "Adapter returned no bundle" });
      return;
    }

    // TODO: validate bundle (schema, minRuntimeVersion), hydrate stores, set initial context
    this.emit("bundle:loaded", { contextId: this.currentContextId });
  }

  setContext(contextId: string | null): void {
    // TODO: verify context exists; update internal state/store views only
    this.currentContextId = contextId;
    this.emit("context:changed", { contextId });
  }

  getCurrentContext(): string | null {
    return this.currentContextId;
  }

  getVisibleNodes(): ReadonlyArray<unknown> {
    // TODO: return nodes visible in current context
    return [];
  }

  preloadAssets(): Promise<void> {
    // TODO: implement once asset strategy is added
    return Promise.resolve();
  }

  on<K extends EventKey>(type: K, listener: Listener<K>): () => void {
    let set = this.listeners.get(type);
    if (!set) {
      set = new Set();
      this.listeners.set(type, set);
    }
    set.add(listener as Function);
    return () => this.off(type, listener);
  }

  off<K extends EventKey>(type: K, listener: Listener<K>): void {
    const set = this.listeners.get(type);
    if (set) {
      set.delete(listener as Function);
    }
  }

  private emit<K extends EventKey>(type: K, payload: KartaRuntimeEvents[K]): void {
    const set = this.listeners.get(type);
    if (!set || set.size === 0) return;
    for (const fn of set) {
      try {
        (fn as Listener<K>)(payload);
      } catch {
        // swallow; also emit error
      }
    }
  }

  destroy(): void {
    // Clean up any internal resources; keeps no-op for now
    this.listeners.clear();
    this.currentContextId = null;
  }
}