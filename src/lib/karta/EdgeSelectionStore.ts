import { writable } from 'svelte/store';
import type { EdgeId } from '$lib/types/types';

// Store to hold the IDs of currently selected edges
export const selectedEdgeIds = writable<Set<EdgeId>>(new Set());

// Action to clear the current edge selection
export function clearEdgeSelection() {
    selectedEdgeIds.set(new Set());
}

// Action to set the selected edges (replaces current selection)
export function setSelectedEdges(edgeIds: EdgeId[]) {
    selectedEdgeIds.set(new Set(edgeIds));
}

// Action to toggle the selection state of a single edge
export function toggleEdgeSelection(edgeId: EdgeId, add: boolean = false, subtract: boolean = false) {
	selectedEdgeIds.update(currentSelection => {
		if (add) {
			const newSelection = new Set(currentSelection);
			newSelection.add(edgeId);
			return newSelection;
		}
		if (subtract) {
			const newSelection = new Set(currentSelection);
			newSelection.delete(edgeId);
			return newSelection;
		}
		// Default behavior: if the edge is already the only thing selected, deselect it.
		// Otherwise, select only this edge.
		if (currentSelection.has(edgeId) && currentSelection.size === 1) {
			return new Set();
		} else {
			return new Set([edgeId]);
		}
	});
}

// Action to deselect a single edge
export function deselectEdge(edgeId: EdgeId) {
    selectedEdgeIds.update(currentSelection => {
        const newSelection = new Set(currentSelection);
        newSelection.delete(edgeId);
        return newSelection;
    });
}