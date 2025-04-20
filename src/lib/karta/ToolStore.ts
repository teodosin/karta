// ToolStore: Manages the active tool, connection process state, and related actions.

import { writable, get } from 'svelte/store';
import type { NodeId, Tool } from '../types/types';
import { MoveTool } from '../tools/MoveTool';
import { ConnectTool } from '../tools/ConnectTool';
import { ContextTool } from '../tools/ContextTool';
// Import necessary stores when they are fully defined
// import { currentViewNodes } from './ContextStore'; // Placeholder
// import { createEdge } from './EdgeStore'; // Placeholder

// --- Tool State ---
const toolInstances = { move: new MoveTool(), connect: new ConnectTool(), context: new ContextTool() };
export const currentTool = writable<Tool>(toolInstances.move); // Default to MoveTool

// --- Connection State ---
export const isConnecting = writable<boolean>(false);
export const connectionSourceNodeIds = writable<NodeId[]>([]);
export const tempLineTargetPosition = writable<{ x: number; y: number } | null>(null);

// --- Tool Actions ---
export function setTool(toolName: 'move' | 'connect' | 'context') {
	const current = get(currentTool);
	const next = toolInstances[toolName];
	if (current !== next) {
		current?.deactivate();
		next?.activate();
		currentTool.set(next);
		console.log(`Switched tool to: ${toolName}`);
	}
}

// --- Connection Actions ---
export function startConnectionProcess(sourceIds: NodeId[]) {
	if (get(isConnecting) || sourceIds.length === 0) return; // Prevent starting if already connecting or no sources

	// TODO: Replace with import from ContextStore when available
	const currentViewNodes_placeholder = new Map(); // Placeholder
	// const nodes = get(currentViewNodes); // Use derived store from ContextStore eventually

	// Check if all source nodes exist in the current view (optional, but good practice)
	// const allSourcesExist = sourceIds.every(id => nodes.has(id));
	const allSourcesExist = true; // Assume true for now

	if (allSourcesExist) {
		isConnecting.set(true);
		connectionSourceNodeIds.set(sourceIds); // Set the array of source IDs
		tempLineTargetPosition.set(null); // Ensure it starts null
		console.log(`[ToolStore] Starting connection from node(s): ${sourceIds.join(', ')}`);
	} else {
		console.warn(`[ToolStore] Could not start connection: One or more source nodes not found in current context.`);
	}
}

export function updateTempLinePosition(canvasX: number, canvasY: number) {
	if (get(isConnecting)) {
		tempLineTargetPosition.set({ x: canvasX, y: canvasY });
	}
}

export function finishConnectionProcess(targetNodeId: NodeId | null) {
	if (!get(isConnecting)) return;
	const sourceIds = get(connectionSourceNodeIds);

	if (targetNodeId) {
		// TODO: Replace with import from ContextStore when available
		const currentViewNodes_placeholder = new Map(); // Placeholder
		// const nodes = get(currentViewNodes); // Use derived store from ContextStore eventually

		// Check if target node exists (optional but good practice)
		// if (nodes.has(targetNodeId)) {
		if (true) { // Assume true for now
			console.log(`[ToolStore] Finishing connection to target node: ${targetNodeId}`);
			// Create edge(s)
			sourceIds.forEach(sourceId => {
				if (sourceId !== targetNodeId) { // Prevent self-connection for each source
					// TODO: Replace with import from EdgeStore when available
					console.log(`[ToolStore] Placeholder: Would call createEdge(${sourceId}, ${targetNodeId})`);
					// createEdge(sourceId, targetNodeId); // Call action from EdgeStore eventually
				} else {
					console.log(`[ToolStore] Skipping self-connection for node: ${sourceId}`);
				}
			});
		} else {
			console.log(`[ToolStore] Connection cancelled: Target node ${targetNodeId} not found.`);
		}
	} else {
		console.log("[ToolStore] Connection cancelled.");
	}

	isConnecting.set(false);
	connectionSourceNodeIds.set([]); // Reset to empty array
	tempLineTargetPosition.set(null);
}

export function cancelConnectionProcess() {
	finishConnectionProcess(null);
}

// Activate the default tool on initialization (if needed, or handle in main init)
// get(currentTool)?.activate();