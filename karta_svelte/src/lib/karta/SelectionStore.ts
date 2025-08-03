import { writable, get } from 'svelte/store';
import type { NodeId } from '../types/types';

export const selectedNodeIds = writable<Set<NodeId>>(new Set());

/** Clears the current selection. */
export function clearSelection() {
selectedNodeIds.update(currentSelection => {
    if (currentSelection.size > 0) {
        return new Set<NodeId>(); // Return new Set to trigger update
    }
    return currentSelection; // No change if already empty
});
}

/**
* Sets the selection to the provided node IDs.
* @param nodeIds A single NodeId or an array/Set of NodeIds.
*/
export function setSelectedNodes(nodeIds: NodeId | NodeId[] | Set<NodeId>) {
const idsToSelect = new Set(Array.isArray(nodeIds) ? nodeIds : nodeIds instanceof Set ? Array.from(nodeIds) : [nodeIds]);
selectedNodeIds.set(idsToSelect);
}


/** Deselects a specific node. */
export function deselectNode(nodeId: NodeId) {
	selectedNodeIds.update(currentSelection => {
		if (!currentSelection.has(nodeId)) {
			return currentSelection; // No change needed
		}
		// Create a new Set without the specified nodeId
		const nextSelection = new Set(currentSelection);
		nextSelection.delete(nodeId);
		return nextSelection; // Return the new Set
	});
}

/** Toggles the selection state of a single node. */
export function toggleSelection(nodeId: NodeId) {
	selectedNodeIds.update(currentSelection => {
		// Create a new Set based on the current one
		const nextSelection = new Set(currentSelection);
		if (nextSelection.has(nodeId)) {
			nextSelection.delete(nodeId); // Modify the new Set
		} else {
			nextSelection.add(nodeId); // Modify the new Set
		}
		return nextSelection; // Return the new Set
	});
}