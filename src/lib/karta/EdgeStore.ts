import { writable, get } from 'svelte/store';
import { activeAdapter } from './ContextStore';
import type { KartaEdge, EdgeId, NodeId, KartaEdgeCreationPayload } from '../types/types';
import { nodes } from './NodeStore';
import { v4 as uuidv4 } from 'uuid';

export const edges = writable<Map<EdgeId, KartaEdge>>(new Map());

export async function createEdges(sourceIds: NodeId[], targetId: NodeId) {
    const allNodes = get(nodes);
    const currentEdges = get(edges);
    const newEdges: KartaEdgeCreationPayload[] = [];

    const targetNode = allNodes.get(targetId);
    if (!targetNode) {
        console.error(`[createEdges] Target node ${targetId} not found in store.`);
        return;
    }

    for (const sourceId of sourceIds) {
        if (sourceId === targetId) {
            console.warn("Cannot connect node to itself.");
            continue;
        }

        const sourceNode = allNodes.get(sourceId);
        if (!sourceNode) {
            console.error(`[createEdges] Source node ${sourceId} not found in store.`);
            continue;
        }

        const edgeExists = Array.from(currentEdges.values()).some(
            (edge) =>
                (edge.source === sourceId && edge.target === targetId) ||
                (edge.source === targetId && edge.target === sourceId)
        );
        if (edgeExists) {
            console.warn(`Edge between ${sourceId} and ${targetId} already exists.`);
            continue;
        }

        const newEdge: KartaEdgeCreationPayload = {
            id: uuidv4(),
            source: sourceId,
            target: targetId,
            attributes: {},
            source_path: sourceNode.path,
            target_path: targetNode.path,
        };
        newEdges.push(newEdge);
    }

    if (newEdges.length === 0) return;

    edges.update((e) => {
        for (const edge of newEdges) {
            e.set(edge.id, edge);
        }
        return e;
    });

    if (activeAdapter) {
        try {
            console.log('[EdgeStore.createEdges] Payload to be sent to adapter:', JSON.stringify(newEdges, null, 2));
            await activeAdapter.createEdges(newEdges);
        } catch (error) {
            console.error("Error saving edges:", error);
            // TODO: Add error handling, maybe revert the store update
        }
    } else {
        console.warn("ActiveAdapter not initialized, persistence disabled.");
    }
}

export async function deleteEdges(edgeIds: EdgeId[]) {
    edges.update(e => {
        for (const id of edgeIds) {
            e.delete(id);
        }
        return e;
    });
    if (activeAdapter) {
        try {
            await activeAdapter.deleteEdges(edgeIds);
        } catch (error) {
            console.error("Error deleting edges:", error);
            // TODO: Add error handling, maybe revert the store update
        }
    } else {
        console.warn("ActiveAdapter not initialized, persistence disabled.");
    }
}