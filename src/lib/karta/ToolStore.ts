import { writable, get } from 'svelte/store';
import type { KartaEdge, NodeId, Tool } from '../types/types';
import { MoveTool } from '../tools/MoveTool';
import { ConnectTool } from '../tools/ConnectTool';
import { ContextTool } from '../tools/ContextTool';
import { createEdges, edges as edgeStore, reconnectEdge } from './EdgeStore';
import { currentViewNodes, currentContextId } from './ContextStore';



// Stores
export const currentTool 				= writable<Tool>(null as any);
export const isConnecting 				= writable<boolean>(false);
export const connectionSourceNodeIds 	= writable<NodeId[]>([]);
export const tempLineTargetPosition 	= writable<{ x: number; y: number } | null>(null);

// Reconnection Stores
export const isReconnecting = writable<boolean>(false);
export const reconnectingEdgeId = writable<NodeId | null>(null);
export const reconnectingEndpoint = writable<'from' | 'to' | null>(null);


// Tool Management
let instantiatedToolInstances: { [key: string]: Tool } | null = null;





export function setTool(toolName: 'move' | 'connect' | 'context') {
    // Ensure tools are initialized before setting
    if (!instantiatedToolInstances) {
        // console.error("Tool instances not initialized yet!"); // Keep error logs for now
        return;
    }
    const current = get(currentTool);
    const next = instantiatedToolInstances[toolName];
    if (current !== next) {
        current?.deactivate(); next?.activate(); currentTool.set(next);
    }
}





// Connection Process Helpers
export function startConnectionProcess(sourceIds: NodeId[]) {
	// Removed tool check: if (get(currentTool)?.name !== 'connect' || get(isConnecting)) return;
	if (get(isConnecting) || sourceIds.length === 0) return; // Prevent starting if already connecting or no sources
	const contextId = get(currentContextId);
	const nodes = get(currentViewNodes); // Use currentViewNodes for simpler access
	// Check if all source nodes exist in the current view (optional, but good practice)
	const allSourcesExist = sourceIds.every(id => nodes.has(id));
	if (allSourcesExist) {
		isConnecting.set(true);
		connectionSourceNodeIds.set(sourceIds); // Set the array of source IDs
		// Initialize temp line position slightly offset from the first source node? Or handle in EdgeLayer?
		// For now, let EdgeLayer handle initial position based on source node centers.
		tempLineTargetPosition.set(null); // Ensure it starts null
	} else {
		console.warn(`[KartaStore] Could not start connection: One or more source nodes not found in context ${contextId}`);
	}
}





export function updateTempLinePosition(canvasX: number, canvasY: number) {

	if (get(isConnecting) || get(isReconnecting)) {
		tempLineTargetPosition.set({ x: canvasX, y: canvasY });
	}
}





export function finishConnectionProcess(targetNodeId: NodeId | null) {

	if (!get(isConnecting)) return;

	const sourceIds = get(connectionSourceNodeIds);

	if (targetNodeId) {
		// Check if target node exists (optional but good practice)
		const nodes = get(currentViewNodes); // Use currentViewNodes
		if (nodes.has(targetNodeId)) {
			// Create edge(s)
			createEdges(sourceIds, targetNodeId);
			// Note: The original 'else' block after createEdge seemed misplaced/empty, removed it.
		} else {
		}
	}
	else { /* Connection cancelled */ }
	isConnecting.set(false);
	connectionSourceNodeIds.set([]); // Reset to empty array
	tempLineTargetPosition.set(null);
}





export function cancelConnectionProcess() {
	
	finishConnectionProcess(null);
}

// Reconnection Process Helpers
export function startReconnectionProcess(edgeId: NodeId, endpoint: 'from' | 'to') {
	if (get(isReconnecting)) return;

	const edge = get(edgeStore).get(edgeId);
	if (!edge) {
		console.warn(`[ToolStore] Could not start reconnection: Edge ${edgeId} not found.`);
		return;
	}

	isReconnecting.set(true);
	reconnectingEdgeId.set(edgeId);
	reconnectingEndpoint.set(endpoint);
	tempLineTargetPosition.set(null); // Ensure it starts null
}

export function finishReconnectionProcess(targetNodeId: NodeId | null) {
	if (!get(isReconnecting)) return;

	const edgeId = get(reconnectingEdgeId);
	const endpoint = get(reconnectingEndpoint);
	const allEdges = get(edgeStore);
	const edgeToReconnect = edgeId ? allEdges.get(edgeId) : null;

	if (targetNodeId && edgeId && endpoint && edgeToReconnect) {
		// Rule: Prevent reconnecting 'contains' edges
		if (edgeToReconnect.contains) {
			console.warn('[ToolStore] Reconnecting "contains" edges is not allowed.');
			cancelReconnectionProcess();
			return;
		}

		const currentSource = edgeToReconnect.source;
		const currentTarget = edgeToReconnect.target;

		const newSource = endpoint === 'from' ? targetNodeId : currentSource;
		const newTarget = endpoint === 'to' ? targetNodeId : currentTarget;

		// Rule: Prevent self-connections
		if (newSource === newTarget) {
			console.warn('[ToolStore] Cannot reconnect an edge to the same node.');
			cancelReconnectionProcess();
			return;
		}

		// Rule: Prevent creating a duplicate edge
		const edgeExists = Array.from(allEdges.values()).some(
			(edge) =>
				(edge.source === newSource && edge.target === newTarget) ||
				(edge.source === newTarget && edge.target === newSource)
		);

		if (edgeExists) {
			console.warn(
				`[ToolStore] An edge between ${newSource} and ${newTarget} already exists.`
			);
			cancelReconnectionProcess();
			return;
		}

		// All checks passed, proceed with reconnection
		console.log(`[ToolStore] Reconnect edge ${edgeId} (${endpoint}) to node ${targetNodeId}`);
		if (endpoint === 'from') {
			reconnectEdge(edgeId, targetNodeId, undefined);
		} else {
			reconnectEdge(edgeId, undefined, targetNodeId);
		}
	}

	cancelReconnectionProcess();
}

export function cancelReconnectionProcess() {
	isReconnecting.set(false);
	reconnectingEdgeId.set(null);
	reconnectingEndpoint.set(null);
	tempLineTargetPosition.set(null);
}



/**
	Main initializer for tool instances.
*/
export function initializeTools() {
    instantiatedToolInstances = {
        move: new MoveTool(),
        connect: new ConnectTool(),
        context: new ContextTool()
    };

    currentTool.set(instantiatedToolInstances.move);
}