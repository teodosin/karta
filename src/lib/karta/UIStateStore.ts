import { writable, get } from 'svelte/store';
import type { NodeId } from '../types/types';
import { createNodeAtPosition } from './NodeStore'; // Assuming NodeStore exports createNodeAtPosition
import { getDefaultAttributesForType } from '$lib/node_types/registry'; // Assuming this is needed here

// Create Node Menu Stores
export const isCreateNodeMenuOpen = writable<boolean>(false);
export const createNodeMenuPosition = writable<{ screenX: number; screenY: number; canvasX: number; canvasY: number } | null>(null);

// Context Menu Stores
export const isContextMenuOpen = writable<boolean>(false);
export const contextMenuPosition = writable<{ x: number; y: number } | null>(null); // Screen coordinates
export type ContextMenuContextType = { type: 'node' | 'edge' | 'background'; id?: string };
export const contextMenuContext = writable<ContextMenuContextType | null>(null);

// Properties Panel Stores
export const propertiesPanelVisible = writable<boolean>(false);
export const propertiesPanelNodeId = writable<NodeId | null>(null);
export const propertiesPanelPosition = writable<{ x: number; y: number }>({ x: 0, y: 0 }); // Initial safe default
export const propertiesPanelSize = writable<{ width: number; height: number }>({ width: 300, height: 400 }); // Default size
export const propertiesPanelCollapsed = writable<boolean>(false);

// Rename Request State
export const nodeRenameRequestId = writable<NodeId | null>(null);


// Create Node Menu Actions
export function openCreateNodeMenu(screenX: number, screenY: number, canvasX: number, canvasY: number) {
    createNodeMenuPosition.set({ screenX, screenY, canvasX, canvasY });
    isCreateNodeMenuOpen.set(true);
    console.log(`[KartaStore] Opening create node menu at (${screenX}, ${screenY})`);
}

export function closeCreateNodeMenu() {
    isCreateNodeMenuOpen.set(false);
    createNodeMenuPosition.set(null);
    console.log(`[KartaStore] Closing create node menu`);
}

export async function createNodeFromMenu(ntype: string) {
    const position = get(createNodeMenuPosition);
    const viewportEl = document.getElementById('viewport'); // Assuming viewport has this ID

    // Use the canvas coordinates directly from the stored position
    if (position) {
        // Get default attributes from registry for the selected type
        const defaultAttributes = getDefaultAttributesForType(ntype);
        // Use stored canvasX, canvasY
        await createNodeAtPosition(position.canvasX, position.canvasY, ntype, defaultAttributes);
        closeCreateNodeMenu(); // Close menu after successful creation attempt
    } else if (viewportEl) { // Added check for viewportEl for error message context
        console.error('[KartaStore] Cannot create node from menu: Position not found in store.');

        closeCreateNodeMenu(); // Close menu after creation
    } else {
        console.error('[KartaStore] Cannot create node from menu: Viewport element not found.');
        closeCreateNodeMenu(); // Close menu even if creation failed
    }
}

// Context Menu Actions
export function openContextMenu(position: { x: number; y: number }, context: ContextMenuContextType) {
    contextMenuPosition.set(position);
    contextMenuContext.set(context);
    isContextMenuOpen.set(true);
    console.log(`[KartaStore] Opening context menu at (${position.x}, ${position.y}) for context:`, context);
}

export function closeContextMenu() {
    isContextMenuOpen.set(false);
    contextMenuPosition.set(null);
    contextMenuContext.set(null);
    console.log(`[KartaStore] Closing context menu`);
}

// Properties Panel Actions
export function setPropertiesPanelVisibility(visible: boolean) {
    propertiesPanelVisible.set(visible);
}

export function setPropertiesPanelNode(nodeId: NodeId | null) {
    propertiesPanelNodeId.set(nodeId);
    // Automatically show panel when a node is set, hide when null? Or handle in subscription?
    // Let's handle visibility explicitly via subscription for now.
}

export function setPropertiesPanelPosition(pos: { x: number; y: number }) {
    propertiesPanelPosition.set(pos);
}

export function setPropertiesPanelSize(size: { width: number; height: number }) {
    propertiesPanelSize.set(size);
}

export function togglePropertiesPanelCollapsed() {
    console.log('PANEL: ', get(propertiesPanelCollapsed));
    propertiesPanelCollapsed.update(collapsed => !collapsed);
    console.log('PANEL: ', get(propertiesPanelCollapsed));
}

/** Signals that a rename should be initiated for the specified node. */
export function requestNodeRename(nodeId: NodeId) {
    // This function will likely need to interact with NodeStore to check if the node is a system node
    // For now, just setting the request ID.
    nodeRenameRequestId.set(nodeId);
    console.log(`[KartaStore] Requested rename for node ${nodeId}`);
}
// --- Subscription to link selection changes to properties panel visibility ---
import { selectedNodeIds } from './SelectionStore';

selectedNodeIds.subscribe(selectedIds => {
	console.log(`[UIStateStore Sub] selectedNodeIds changed. Size: ${selectedIds.size}`);
	if (selectedIds.size === 1) {
		const selectedId = selectedIds.values().next().value;
		console.log(`[UIStateStore Sub] Exactly one node selected: ${selectedId}. Setting properties panel node and visibility to true.`);
		setPropertiesPanelNode(selectedId ?? null); // Use nullish coalescing for type safety
		setPropertiesPanelVisibility(true);
	} else {
		console.log(`[UIStateStore Sub] ${selectedIds.size} nodes selected. Setting properties panel node to null and visibility to false.`);
		setPropertiesPanelNode(null);
		setPropertiesPanelVisibility(false);
	}
});