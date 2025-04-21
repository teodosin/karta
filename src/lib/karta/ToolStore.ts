import { writable, get } from 'svelte/store';
import type { Tool, NodeId } from '../types/types';
import { MoveTool } from '../tools/MoveTool';
import { ConnectTool } from '../tools/ConnectTool';
import { ContextTool } from '../tools/ContextTool';
import { createEdge } from './EdgeStore'; // Assuming EdgeStore exports createEdge
import { currentViewNodes, currentContextId } from './ContextStore'; // Assuming ContextStore exports these

// Stores
export const currentTool = writable<Tool>(null as any); // Initialize with null or a dummy tool, will be set in initializeTools
export const isConnecting = writable<boolean>(false);
export const connectionSourceNodeIds = writable<NodeId[]>([]); // Changed to array
export const tempLineTargetPosition = writable<{ x: number; y: number } | null>(null);

// Tool Management
let instantiatedToolInstances: { [key: string]: Tool } | null = null; // Declare and initialize to null

export function setTool(toolName: 'move' | 'connect' | 'context') {
    // Ensure tools are initialized before setting
    if (!instantiatedToolInstances) {
        console.error("Tool instances not initialized yet!");
        return;
    }
    const current = get(currentTool);
    const next = instantiatedToolInstances[toolName];
    if (current !== next) {
        current?.deactivate(); next?.activate(); currentTool.set(next);
        console.log(`Switched tool to: ${toolName}`);
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
		console.log(`[KartaStore] Starting connection from node(s): ${sourceIds.join(', ')}`);
	} else {
		console.warn(`[KartaStore] Could not start connection: One or more source nodes not found in context ${contextId}`);
	}
}
export function updateTempLinePosition(canvasX: number, canvasY: number) { if (get(isConnecting)) tempLineTargetPosition.set({x: canvasX, y: canvasY}); }
export function finishConnectionProcess(targetNodeId: NodeId | null) {
	if (!get(isConnecting)) return;
	const sourceIds = get(connectionSourceNodeIds);
	if (targetNodeId) {
		// Check if target node exists (optional but good practice)
		const nodes = get(currentViewNodes); // Use currentViewNodes
		if (nodes.has(targetNodeId)) {
			console.log(`[KartaStore] Finishing connection to target node: ${targetNodeId}`);
			// Create edge(s)
			sourceIds.forEach(sourceId => {
				if (sourceId !== targetNodeId) { // Prevent self-connection for each source
					createEdge(sourceId, targetNodeId); // Call existing createEdge
				} else {
					console.log(`[KartaStore] Skipping self-connection for node: ${sourceId}`);
				}
			});
			// Note: The original 'else' block after createEdge seemed misplaced/empty, removed it.
		} else {
			console.log(`[KartaStore] Connection cancelled: Target node ${targetNodeId} not found.`);
		}
	}
	else console.log("Connection cancelled.");
	isConnecting.set(false);
	connectionSourceNodeIds.set([]); // Reset to empty array
	tempLineTargetPosition.set(null);
}
export function cancelConnectionProcess() { finishConnectionProcess(null); }

// Function to initialize tool instances
export function initializeTools() {
    instantiatedToolInstances = {
        move: new MoveTool(),
        connect: new ConnectTool(),
        context: new ContextTool()
    };
    // Set the initial tool after instantiation
    currentTool.set(instantiatedToolInstances.move);
    console.log("[ToolStore] Tool instances initialized.");
}