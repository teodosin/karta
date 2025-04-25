import { writable, get } from 'svelte/store';
import { localAdapter } from '../util/LocalAdapter';
import type { KartaEdge, EdgeId, NodeId } from '../types/types';
import { v4 as uuidv4 } from 'uuid';

export const edges = writable<Map<EdgeId, KartaEdge>>(new Map());

export async function createEdge(sourceId: NodeId, targetId: NodeId) {
    if (sourceId === targetId) { console.warn("Cannot connect node to itself."); return; }
    const currentEdges = get(edges);
    for (const edge of currentEdges.values()) {
        if ((edge.source === sourceId && edge.target === targetId) || (edge.source === targetId && edge.target === sourceId)) {
            console.warn(`Edge between ${sourceId} and ${targetId} already exists.`); return;
        }
    }
    const newEdgeId: EdgeId = uuidv4();
	const newEdge: KartaEdge = { id: newEdgeId, source: sourceId, target: targetId };
	edges.update(e => e.set(newEdgeId, newEdge));
	console.log(`Created edge ${newEdgeId} between ${sourceId} and ${targetId}`);
    if (localAdapter) {
        try { await localAdapter.saveEdge(newEdge); }
        catch (error) { console.error("Error saving edge:", error); }
    } else { console.warn("LocalAdapter not initialized, persistence disabled."); }
}

export async function deleteEdge(edgeId: EdgeId) {
    edges.update(e => {
        e.delete(edgeId);
        return e;
    });
    console.log(`Deleted edge ${edgeId}`);
    if (localAdapter) {
        try { await localAdapter.deleteEdge(edgeId); }
        catch (error) { console.error("Error deleting edge:", error); }
    } else { console.warn("LocalAdapter not initialized, persistence disabled."); }
}