// UIStateStore: Manages state for UI elements like menus, panels.

import { writable, get } from 'svelte/store';
import type { NodeId } from '../types/types';

// --- Create Node Menu State ---
export const isCreateNodeMenuOpen = writable<boolean>(false);
export const createNodeMenuPosition = writable<{ screenX: number; screenY: number; canvasX: number; canvasY: number } | null>(null);

// --- Context Menu State ---
export const isContextMenuOpen = writable<boolean>(false);
export const contextMenuPosition = writable<{ x: number; y: number } | null>(null); // Screen coordinates
export type ContextMenuContextType = { type: 'node' | 'edge' | 'background'; id?: string };
export const contextMenuContext = writable<ContextMenuContextType | null>(null);

// --- Properties Panel State ---
export const propertiesPanelVisible = writable<boolean>(false);
export const propertiesPanelNodeId = writable<NodeId | null>(null);
export const propertiesPanelPosition = writable<{ x: number; y: number }>({ x: 0, y: 0 }); // Initial safe default
export const propertiesPanelSize = writable<{ width: number; height: number }>({ width: 300, height: 400 }); // Default size
export const propertiesPanelCollapsed = writable<boolean>(false);

// --- Rename Request State ---
export const nodeRenameRequestId = writable<NodeId | null>(null);

// --- UI Actions ---

// Create Node Menu Actions
export function openCreateNodeMenu(screenX: number, screenY: number, canvasX: number, canvasY: number) {
	createNodeMenuPosition.set({ screenX, screenY, canvasX, canvasY });
	isCreateNodeMenuOpen.set(true);
	console.log(`[UIStateStore] Opening create node menu at (${screenX}, ${screenY})`);
}

export function closeCreateNodeMenu() {
	isCreateNodeMenuOpen.set(false);
	createNodeMenuPosition.set(null);
	console.log(`[UIStateStore] Closing create node menu`);
}

// Context Menu Actions
export function openContextMenu(position: { x: number; y: number }, context: ContextMenuContextType) {
	contextMenuPosition.set(position);
	contextMenuContext.set(context);
	isContextMenuOpen.set(true);
	console.log(`[UIStateStore] Opening context menu at (${position.x}, ${position.y}) for context:`, context);
}

export function closeContextMenu() {
	isContextMenuOpen.set(false);
	contextMenuPosition.set(null);
	contextMenuContext.set(null);
	console.log(`[UIStateStore] Closing context menu`);
}

// Properties Panel Actions
export function setPropertiesPanelVisibility(visible: boolean) {
	propertiesPanelVisible.set(visible);
}

export function setPropertiesPanelNode(nodeId: NodeId | null) {
	propertiesPanelNodeId.set(nodeId);
	// Visibility is handled by subscription in the original KartaStore,
	// that logic will need to be moved or replicated elsewhere (e.g., in a component or main init).
}

export function setPropertiesPanelPosition(pos: { x: number; y: number }) {
	propertiesPanelPosition.set(pos);
}

export function setPropertiesPanelSize(size: { width: number; height: number }) {
	propertiesPanelSize.set(size);
}

export function togglePropertiesPanelCollapsed() {
	propertiesPanelCollapsed.update(collapsed => !collapsed);
	console.log('[UIStateStore] Toggled properties panel collapsed state:', get(propertiesPanelCollapsed));
}

// Rename Request Action
/** Signals that a rename should be initiated for the specified node. */
export function requestNodeRename(nodeId: NodeId) {
	// TODO: Add check for system node (requires importing NodeStore)
	// const node = get(nodes).get(nodeId); // From NodeStore
	// if (node && !node.attributes?.isSystemNode) {
	nodeRenameRequestId.set(nodeId);
	console.log(`[UIStateStore] Requested rename for node ${nodeId}`);
	// } else if (node?.attributes?.isSystemNode) {
	// 	console.warn(`[UIStateStore] Rename request ignored for system node ${nodeId}`);
	// } else {
	// 	console.warn(`[UIStateStore] Rename request ignored for non-existent node ${nodeId}`);
	// }
}

// Set initial panel position (requires browser context)
// This should likely move to the main initialization logic
// if (typeof window !== 'undefined') {
// 	propertiesPanelPosition.set({ x: window.innerWidth - 320, y: 50 });
// }