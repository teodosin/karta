// EdgeStore: Manages KartaEdge state and related actions.

import { writable, get } from 'svelte/store';
import type { EdgeId, KartaEdge, NodeId } from '../types/types';
import { v4 as uuidv4 } from 'uuid';
import { localAdapter } from '../util/LocalAdapter'; // Assuming LocalAdapter is initialized elsewhere

export const edges = writable<Map<EdgeId, KartaEdge>>(new Map());

// --- Public Actions ---

// Edge Creation
export async function createEdge(sourceId: NodeId, targetId: NodeId) {
	if (sourceId === targetId) {
		console.warn("[EdgeStore] Cannot connect node to itself.");
		return;
	}
	const currentEdges = get(edges);
	// Check if edge already exists (in either direction)
	for (const edge of currentEdges.values()) {
		if ((edge.source === sourceId && edge.target === targetId) || (edge.source === targetId && edge.target === sourceId)) {
			console.warn(`[EdgeStore] Edge between ${sourceId} and ${targetId} already exists.`);
			return;
		}
	}

	const newEdgeId: EdgeId = uuidv4();
	const newEdge: KartaEdge = { id: newEdgeId, source: sourceId, target: targetId };

	// Update store
	edges.update(e => e.set(newEdgeId, newEdge));
	console.log(`[EdgeStore] Created edge ${newEdgeId} between ${sourceId} and ${targetId}`);

	// Persist changes
	if (localAdapter) {
		try {
			await localAdapter.saveEdge(newEdge);
		} catch (error) {
			console.error("[EdgeStore] Error saving edge:", error);
			// Revert store update on save failure?
			edges.update(e => { e.delete(newEdgeId); return e; });
		}
	} else {
		console.warn("[EdgeStore] LocalAdapter not initialized, persistence disabled.");
	}
}

// TODO: Add edge deletion action if needed