// initialize.ts: Handles the initial setup of Karta stores on application load.

import { get, writable } from 'svelte/store'; // Added writable import
import { localAdapter } from '../util/LocalAdapter';
import type { DataNode, NodeId, ViewportSettings } from '../types/types';
// Removed unused type imports: Context, EdgeId, KartaEdge, TweenableNodeState, getDefaultViewNodeStateForType

// Import stores and actions
import { nodes as nodeStore, _ensureDataNodeExists } from './NodeStore';
import { edges as edgeStore } from './EdgeStore'; // Keep for setting empty state on error
import {
	contexts as contextStore, // Keep for setting empty state on error
	currentContextId,
	ROOT_NODE_ID,
	switchContext // Import the public action
} from './ContextStore';
import { viewTransform } from './ViewportStore';
import { currentTool, setTool } from './ToolStore';
import { propertiesPanelPosition } from './UIStateStore';

// Constants
const DEFAULT_VIEWPORT_SETTINGS: ViewportSettings = { scale: 1, posX: 0, posY: 0 };

export async function initializeStores() {
	console.log("[initializeStores] Initializing Karta stores...");

	if (!localAdapter) {
		console.error("[initializeStores] LocalAdapter not initialized. Cannot proceed.");
		// Set default empty state on critical error
		nodeStore.set(new Map());
		edgeStore.set(new Map());
		contextStore.set(new Map());
		currentContextId.set(ROOT_NODE_ID);
		viewTransform.set(DEFAULT_VIEWPORT_SETTINGS, { duration: 0 });
		setTool('move');
		return;
	}

	// Activate the default tool
	get(currentTool)?.activate();

	let targetInitialContextId = ROOT_NODE_ID;
	let initialDataNode: DataNode | null = null;

	try {
		// 1. Determine Initial Context ID (Forcing ROOT for now)
		targetInitialContextId = ROOT_NODE_ID;
		console.log(`[initializeStores] Forcing start with ROOT_NODE_ID: ${targetInitialContextId}`);

		// 2. Ensure Initial DataNode Exists
		initialDataNode = await _ensureDataNodeExists(targetInitialContextId);

		// 3. Fallback to Root if target ID failed
		if (!initialDataNode) {
			console.warn(`[initializeStores] Failed to ensure target node ${targetInitialContextId} exists. Falling back to ROOT.`);
			targetInitialContextId = ROOT_NODE_ID;
			initialDataNode = await _ensureDataNodeExists(ROOT_NODE_ID);
			if (!initialDataNode) {
				throw new Error("CRITICAL: Failed to ensure even the Root DataNode exists during initialization.");
			}
		}
		// 3.5 Ensure Root Node has isSystemNode flag (handled by _ensureDataNodeExists)
		// Re-fetch if necessary to ensure the flag is reflected in our variable
		if (initialDataNode.id === ROOT_NODE_ID && !initialDataNode.attributes?.isSystemNode) {
			console.warn(`[initializeStores] Re-fetching root node after ensuring system flag.`);
			initialDataNode = await _ensureDataNodeExists(ROOT_NODE_ID); // Re-fetch to get updated attributes
			if (!initialDataNode || !initialDataNode.attributes?.isSystemNode) {
				console.error(`[initializeStores] CRITICAL: Failed to verify isSystemNode flag on root node after attempting to add it.`);
			}
		}

		// The initialDataNode should be non-null here due to the throw above
		const finalInitialNodeId = initialDataNode!.id; // Add non-null assertion

		// 4. Trigger Context Switch to Load Initial State
		// switchContext handles loading data, setting stores (nodes, edges, contexts, currentContextId), and viewport
		await switchContext(finalInitialNodeId); // Let switchContext handle the initial load

		console.log(`[initializeStores] Stores initialized via switchContext, starting context: ${finalInitialNodeId}`);

	} catch (error) {
		console.error("[initializeStores] Error during store initialization:", error);
		// Set default empty state on error
		nodeStore.set(new Map());
		edgeStore.set(new Map());
		contextStore.set(new Map());
		currentContextId.set(ROOT_NODE_ID);
		viewTransform.set(DEFAULT_VIEWPORT_SETTINGS, { duration: 0 });
	}

	// 5. Set Initial UI State (Post-Initialization)
	if (typeof window !== 'undefined') {
		propertiesPanelPosition.set({ x: window.innerWidth - 320, y: 50 });
	}
	setTool('move'); // Ensure move tool is set

	// 6. Initialize other stores if needed
	// initializeContextStore(); // Example if ContextStore had specific init logic
}

// Optional: Export a flag or promise to indicate initialization status
export const storesInitialized = writable(false);

// Run initialization automatically if in browser
let initPromise: Promise<void> | null = null;
if (typeof window !== 'undefined') {
	initPromise = initializeStores().then(() => {
		storesInitialized.set(true);
		console.log("[initializeStores] Initialization complete.");
	}).catch(err => {
		console.error("[initializeStores] Initialization failed:", err);
	});
}

export { initPromise };