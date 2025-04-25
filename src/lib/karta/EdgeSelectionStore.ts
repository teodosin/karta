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
        const newSelection = new Set(currentSelection);
        if (add) {
            newSelection.add(edgeId);
        } else if (subtract) {
            newSelection.delete(edgeId);
        } else {
            // Default behavior: toggle
            if (newSelection.has(edgeId)) {
                newSelection.delete(edgeId);
            } else {
                newSelection.add(edgeId);
            }
        }
        return newSelection;
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