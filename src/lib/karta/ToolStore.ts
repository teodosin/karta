import { writable, get } from 'svelte/store';
import type { KartaEdge, NodeId, Tool } from '../types/types';
import { MoveTool } from '../tools/MoveTool';
import { ConnectTool } from '../tools/ConnectTool';
import { ContextTool } from '../tools/ContextTool';
import { createEdges, edges as edgeStore, reconnectEdge } from './EdgeStore';
import { currentViewNodes, currentContextId, activeAdapter } from './ContextStore';
import { nodes } from './NodeStore';



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

	// Prevent dragging contains edges from the target (child) end
	if (edge.contains && endpoint === 'to') {
		console.log(`[ToolStore] Contains edges cannot be reconnected from the target (child) side.`);
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

		// Handle contains edges differently - use move nodes API
		if (edgeToReconnect.contains) {
			console.log(`[ToolStore] Moving node ${currentTarget} to new parent ${newSource}`);
			handleContainsEdgeReconnection(currentTarget, newSource);
		} else {
			// Regular edge reconnection
			console.log(`[ToolStore] Reconnect edge ${edgeId} (${endpoint}) to node ${targetNodeId}`);
			if (endpoint === 'from') {
				reconnectEdge(edgeId, targetNodeId, undefined);
			} else {
				reconnectEdge(edgeId, undefined, targetNodeId);
			}
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

async function handleContainsEdgeReconnection(nodeId: NodeId, newParentId: NodeId) {
	const dataNodes = get(nodes);
	const edgeId = get(reconnectingEdgeId);
	
	if (!activeAdapter) {
		console.error('[ToolStore] No active adapter available for moving nodes');
		return;
	}

	if (!edgeId) {
		console.error('[ToolStore] No edge ID available for contains edge reconnection');
		return;
	}

	const node = dataNodes.get(nodeId);
	const newParent = dataNodes.get(newParentId);

	if (!node || !newParent) {
		console.error('[ToolStore] Could not find DataNode or new parent for move operation');
		return;
	}

	if (!node.path || !newParent.path) {
		console.error('[ToolStore] Node or parent missing path information');
		return;
	}

	// Extract node name from current path for building new path
	const nodeName = node.path.split('/').pop();
	if (!nodeName) {
		console.error('[ToolStore] Could not extract node name from path:', node.path);
		return;
	}

	// Calculate the new path: parent_path/node_name
	const newNodePath = `${newParent.path}/${nodeName}`;

	try {
		console.log(`[ToolStore] Moving node ${nodeId} (${node.path}) to parent ${newParentId} (${newParent.path})`);
		
		// Cast activeAdapter to access moveNodes method
		const serverAdapter = activeAdapter as any;
		if (serverAdapter.moveNodes) {
			await serverAdapter.moveNodes([{
				source_path: node.path,
				target_parent_path: newParent.path
			}]);

			console.log('[ToolStore] Node move completed successfully');
			
			// Update the node's path in the NodeStore
			nodes.update((nodeMap) => {
				const nodeToUpdate = nodeMap.get(nodeId);
				if (nodeToUpdate) {
					nodeToUpdate.path = newNodePath;
					nodeMap.set(nodeId, nodeToUpdate);
					console.log(`[ToolStore] Updated node ${nodeId} path to: ${newNodePath}`);
				}
				return nodeMap;
			});
			
			// Update the edge store to reflect the new parent-child relationship
			edgeStore.update((edgeMap) => {
				const edge = edgeMap.get(edgeId!);
				if (edge) {
					edge.source = newParentId;
					edge.target = nodeId;
					edgeMap.set(edgeId!, edge);
					console.log(`[ToolStore] Updated edge ${edgeId} to connect ${newParentId} -> ${nodeId}`);
				}
				return edgeMap;
			});
			
			console.log(`[ToolStore] Move operation completed. Updated node path and edge relationship.`);
		} else {
			console.error('[ToolStore] Adapter does not support moveNodes operation');
		}
	} catch (error) {
		console.error('[ToolStore] Failed to move node:', error);
	}
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