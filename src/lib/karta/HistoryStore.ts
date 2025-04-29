import { writable, get } from 'svelte/store';
import type { NodeId } from '../types/types';
import { switchContext, currentContextId } from './ContextStore'; // Assuming ContextStore exports switchContext

// History Stores
export const historyStack = writable<NodeId[]>([]);
export const futureStack = writable<NodeId[]>([]);

// History Actions
export function undoContextSwitch() {
    const history = get(historyStack);
    if (history.length === 0) {
        return;
    }

    const previousId = history[history.length - 1]; // Get last element
    const currentId = get(currentContextId); // Assuming currentContextId is imported from ContextStore

    historyStack.update(stack => stack.slice(0, -1)); // Remove last element
    futureStack.update(stack => [...stack, currentId]); // Add current to future

    switchContext(previousId, true); // Call switchContext with isUndoRedo flag
}

export function redoContextSwitch() {
    const future = get(futureStack);
    if (future.length === 0) {
        return;
    }

    const nextId = future[future.length - 1]; // Get last element
    const currentId = get(currentContextId); // Assuming currentContextId is imported from ContextStore

    futureStack.update(stack => stack.slice(0, -1)); // Remove last element
    historyStack.update(stack => [...stack, currentId]); // Add current to history

    switchContext(nextId, true); // Call switchContext with isUndoRedo flag
}