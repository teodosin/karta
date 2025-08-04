import { writable, derived, get, type Readable } from "svelte/store";
import type { Context, ContextId } from "../types";

export class BaseContextStore {
  protected _contexts = writable<Map<ContextId, Context>>(new Map());
  protected _currentContext = writable<ContextId>("");

  get contexts(): Readable<Map<ContextId, Context>> {
    return derived(this._contexts, (c: Map<ContextId, Context>) => c);
  }

  get currentContext(): Readable<ContextId> {
    return derived(this._currentContext, (c: ContextId) => c);
  }

  protected setContext(context: Context): void {
    this._contexts.update((contexts: Map<ContextId, Context>) => {
      contexts.set(context.id, context);
      return contexts;
    });
  }

  protected switchContext(contextId: ContextId): void {
    this._currentContext.set(contextId);
  }

  getContext(id: ContextId): Context | undefined {
    return get(this._contexts).get(id);
  }
}