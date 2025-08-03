import { writable, get } from 'svelte/store';
import type { NodeId } from '../types/types';
import { switchContext, currentContextId } from './ContextStore';

// History Stores
export const historyStack = writable<NodeId[]>([]);
export const futureStack = writable<NodeId[]>([]);

// History Actions
export function undoContextSwitch() {

    const history = get(historyStack);
    
    if (history.length === 0) {
        return;
    }

    const previousId = history[history.length - 1];
    const currentId = get(currentContextId);

    historyStack.update(stack => stack.slice(0, -1));
    futureStack.update(stack => [...stack, currentId]);

    switchContext({ type: 'uuid', value: previousId }, true);
}

export function redoContextSwitch() {

    const future = get(futureStack);

    if (future.length === 0) {
        return;
    }

    const nextId = future[future.length - 1];
    const currentId = get(currentContextId);

    futureStack.update(stack => stack.slice(0, -1));
    historyStack.update(stack => [...stack, currentId]);

    switchContext({ type: 'uuid', value: nextId }, true);
}