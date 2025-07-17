import { writable, get } from 'svelte/store';
import type { Tool, NodeId } from '../types/types';
import { MoveTool } from '../tools/MoveTool';
import { ConnectTool } from '../tools/ConnectTool';
import { ContextTool } from '../tools/ContextTool';
import { createEdges } from './EdgeStore';
import { currentViewNodes, currentContextId } from './ContextStore';



// Stores
export const currentTool 				= writable<Tool>(null as any);
export const isConnecting 				= writable<boolean>(false);
export const connectionSourceNodeIds 	= writable<NodeId[]>([]);
export const tempLineTargetPosition 	= writable<{ x: number; y: number } | null>(null);

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

	if (get(isConnecting)) {
		tempLineTargetPosition.set({x: canvasX, y: canvasY});
	}; 
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