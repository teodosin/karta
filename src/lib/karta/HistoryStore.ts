// HistoryStore: Manages context navigation history and undo/redo actions.

import { writable, get } from 'svelte/store';
import type { NodeId } from '../types/types';
// Import necessary stores when they are fully defined
// import { switchContext, currentContextId } from './ContextStore'; // Placeholder

export const historyStack = writable<NodeId[]>([]);
export const futureStack = writable<NodeId[]>([]);

// --- History Actions ---
export function pushHistory(oldContextId: NodeId) {
    historyStack.update(stack => [...stack, oldContextId]);
    futureStack.set([]); // Clear future stack on new action
    console.log(`[HistoryStore] Pushed ${oldContextId} to history.`);
}

export function undoContextSwitch() {
	const history = get(historyStack);
	if (history.length === 0) {
		console.log("[HistoryStore] History stack is empty.");
		return;
	}

	const previousId = history[history.length - 1]; // Get last element
	// TODO: Replace with import from ContextStore when available
	const currentId_placeholder = 'placeholder-current-id';
	// const currentId = get(currentContextId); // From ContextStore

	historyStack.update(stack => stack.slice(0, -1)); // Remove last element
	futureStack.update(stack => [...stack, currentId_placeholder]); // Add current to future

	console.log(`[HistoryStore] Undoing to: ${previousId}`);
	// TODO: Replace with import from ContextStore when available
	console.log(`[HistoryStore] Placeholder: Would call switchContext(${previousId}, true)`);
	// switchContext(previousId, true); // Call switchContext from ContextStore with isUndoRedo flag
}

export function redoContextSwitch() {
	const future = get(futureStack);
	if (future.length === 0) {
		console.log("[HistoryStore] Future stack is empty.");
		return;
	}

	const nextId = future[future.length - 1]; // Get last element
	// TODO: Replace with import from ContextStore when available
	const currentId_placeholder = 'placeholder-current-id';
	// const currentId = get(currentContextId); // From ContextStore

	futureStack.update(stack => stack.slice(0, -1)); // Remove last element
	historyStack.update(stack => [...stack, currentId_placeholder]); // Add current to history

	console.log(`[HistoryStore] Redoing to: ${nextId}`);
	// TODO: Replace with import from ContextStore when available
	console.log(`[HistoryStore] Placeholder: Would call switchContext(${nextId}, true)`);
	// switchContext(nextId, true); // Call switchContext from ContextStore with isUndoRedo flag
}