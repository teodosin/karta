import { writable, get } from 'svelte/store';
import { activeAdapter } from './ContextStore';
import type { KartaEdge, EdgeId, NodeId, KartaEdgeCreationPayload, EdgeDeletionPayload } from '../types/types';
import { nodes, ensureNodesExist } from './NodeStore';
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
            contains: false,
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

export async function reconnectEdge(edgeId: EdgeId, newSourceId?: NodeId, newTargetId?: NodeId) {
	const originalEdge = get(edges).get(edgeId);

	if (!originalEdge) {
		console.error(`[reconnectEdge] Edge ${edgeId} not found.`);
		return;
	}

	const old_from = originalEdge.source;
	const old_to = originalEdge.target;
	const new_from = newSourceId || old_from;
	const new_to = newTargetId || old_to;

	// Ensure the nodes we're connecting to exist in the local store
	await ensureNodesExist([new_from, new_to]);

	const allNodes = get(nodes);
	const newFromNode = allNodes.get(new_from);
	const newToNode = allNodes.get(new_to);

	console.log(`[reconnectEdge] Attempting reconnect. EdgeId: ${edgeId}`);
	console.log(`[reconnectEdge] Old Source: ${old_from}, Old Target: ${old_to}`);
	console.log(`[reconnectEdge] New Source ID: ${new_from}, New Target ID: ${new_to}`);
	console.log(`[reconnectEdge] New Source Node from store:`, JSON.stringify(newFromNode, null, 2));
	console.log(`[reconnectEdge] New Target Node from store:`, JSON.stringify(newToNode, null, 2));

	const new_from_path = newFromNode?.path;
	const new_to_path = newToNode?.path;

	console.log(`[reconnectEdge] New Source Path: ${new_from_path}`);
	console.log(`[reconnectEdge] New Target Path: ${new_to_path}`);

	if ((!new_from_path && new_from_path !== '') || (!new_to_path && new_to_path !== '')) {
		console.error(`[reconnectEdge] Could not find path for new source or target node. Aborting.`);
		// Even after attempting to fetch, if the path is missing, we must exit.
		// This could happen if the node doesn't exist on the server either.
		return;
	}

    // Optimistic Update
    edges.update((e) => {
        const edgeToUpdate = e.get(edgeId);
        if (edgeToUpdate) {
            edgeToUpdate.source = new_from;
            edgeToUpdate.target = new_to;
            e.set(edgeId, edgeToUpdate);
        }
        return e;
    });

    if (activeAdapter) {
        try {
            const newEdge = await activeAdapter.reconnectEdge(old_from, old_to, new_from, new_to, new_from_path, new_to_path);
            if (newEdge) {
                // Final state update
                edges.update((e) => {
                    if (edgeId !== newEdge.id) {
                        e.delete(edgeId);
                    }
                    e.set(newEdge.id, newEdge);
                    return e;
                });
            }
        } catch (error) {
            console.error('Error reconnecting edge:', error);
            // Revert on error
            edges.update((e) => {
                const edgeToRevert = e.get(edgeId);
                if (edgeToRevert) {
                    edgeToRevert.source = old_from;
                    edgeToRevert.target = old_to;
                    e.set(edgeId, edgeToRevert);
                }
                return e;
            });
        }
    } else {
        console.warn('ActiveAdapter not initialized, persistence disabled.');
    }
}

export async function deleteEdges(payloads: EdgeDeletionPayload[]) {
    const edgeIdsToDelete: EdgeId[] = [];
    const currentEdges = get(edges);
    const deletablePayloads: EdgeDeletionPayload[] = [];

    // TODO: Optimize this to avoid multiple loops
    for (const payload of payloads) {
        for (const [id, edge] of currentEdges.entries()) {
            if (
                (edge.source === payload.source && edge.target === payload.target) ||
                (edge.source === payload.target && edge.target === payload.source)
            ) {
                console.log(`[EdgeStore] Checking edge ${id} for deletion:`, edge);
                if (!edge.contains) {
                    edgeIdsToDelete.push(id);
                    deletablePayloads.push(payload);
                }
                break;
            }
        }
    }

    if (edgeIdsToDelete.length === 0) return;

    if (activeAdapter) {
        try {
            // The adapter will now receive the source/target payload
            await activeAdapter.deleteEdges(deletablePayloads);

            // On successful deletion from the server, update the local store
            edges.update(e => {
                for (const id of edgeIdsToDelete) {
                    e.delete(id);
                }
                return e;
            });
        } catch (error) {
            console.error("Error deleting edges:", error);
            // If the server call fails, the local store is not changed.
        }
    } else {
        // If there's no adapter, just update the local store directly.
        edges.update(e => {
            for (const id of edgeIdsToDelete) {
                e.delete(id);
            }
            return e;
        });
        console.warn("ActiveAdapter not initialized, persistence disabled.");
    }
}