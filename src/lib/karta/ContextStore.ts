// ContextStore: Manages Context state (including ViewNodes), currentContextId,
// context switching logic, and ViewNode layout updates.

import { writable, get, derived } from 'svelte/store';
import { Tween } from 'svelte/motion';
import { cubicOut } from 'svelte/easing';
import type {
	NodeId, Context, ViewNode, DataNode, KartaEdge, EdgeId, // Added EdgeId
	TweenableNodeState, AbsoluteTransform, StorableContext,
	StorableViewNode, StorableViewportSettings, ViewportSettings
} from '../types/types';
import { localAdapter } from '../util/LocalAdapter';
import { getDefaultViewNodeStateForType } from '$lib/node_types/registry';

// Import other stores (adjust paths if needed)
import { nodes as nodeStore, _ensureDataNodeExists } from './NodeStore';
import { edges as edgeStore } from './EdgeStore';
import { viewTransform } from './ViewportStore';
import { historyStack, futureStack, pushHistory } from './HistoryStore';
import { clearSelection } from './SelectionStore';

// Constants (Consider moving to a shared constants file)
export const ROOT_NODE_ID = '00000000-0000-0000-0000-000000000000';
const DEFAULT_FOCAL_TRANSFORM: AbsoluteTransform = { x: 0, y: 0, scale: 1 };
const DEFAULT_VIEWPORT_SETTINGS: ViewportSettings = { scale: 1, posX: 0, posY: 0 };
const NODE_TWEEN_DURATION = 250;
const NODE_TWEEN_OPTIONS = { duration: NODE_TWEEN_DURATION, easing: cubicOut };
const VIEWPORT_TWEEN_DURATION = 500; // Keep viewport duration consistent

// --- Core State ---
export const contexts = writable<Map<NodeId, Context>>(new Map());
export const currentContextId = writable<NodeId>(ROOT_NODE_ID);

// --- Derived State ---
export const currentViewNodes = derived(
	[currentContextId, contexts],
	([$currentContextId, $contexts]) => {
		return $contexts.get($currentContextId)?.viewNodes ?? new Map<NodeId, ViewNode>();
	}
);

// --- Internal Helper Functions ---

/**
 * Determines the initial state (position, scale, dimensions, rotation) for a target context's focal node.
 * Uses the existing state if visible in the old context, otherwise calculates defaults.
 */
async function _getFocalNodeInitialState(targetNodeId: NodeId, oldContext: Context | undefined): Promise<TweenableNodeState> {
	const targetViewNodeInOldCtx = oldContext?.viewNodes.get(targetNodeId);

	if (targetViewNodeInOldCtx) {
		// Use existing state from the previous context
		return { ...targetViewNodeInOldCtx.state.current }; // Return a copy
	} else {
		// Node not visible in old context, calculate defaults
		console.log(`[ContextStore - _getFocalNodeInitialState] Node ${targetNodeId} not found in old context. Calculating defaults.`);
		// Use _ensureDataNodeExists from NodeStore
		const dataNode = await _ensureDataNodeExists(targetNodeId);
		const defaultState: { width: number; height: number; scale: number; rotation: number; } = dataNode ? getDefaultViewNodeStateForType(dataNode.ntype) : getDefaultViewNodeStateForType('generic'); // Fallback to generic

		// Combine default placement with default dimensions/rotation
		return {
			...DEFAULT_FOCAL_TRANSFORM, // Default x, y, scale
			width: defaultState.width,
			height: defaultState.height,
			rotation: defaultState.rotation
		};
	}
}

/**
 * Loads a context from DB or creates it, adds default connected nodes (preserving existing positions), saves if modified.
 */
async function _loadAndProcessContext(
	contextId: NodeId,
	focalInitialStateFromOldContext: TweenableNodeState, // Full initial state including dimensions
	oldContext: Context | undefined // Previous context for potential tween reuse
): Promise<{ finalContext: Context | undefined, wasCreated: boolean }> {

	if (!localAdapter) return { finalContext: undefined, wasCreated: false };

	const storableContext = await localAdapter.getContext(contextId);
	let contextWasCreated = false;
	const finalViewNodes = new Map<NodeId, ViewNode>();
	let finalViewportSettings: ViewportSettings | undefined = undefined;

	if (storableContext) {
		// --- Context exists in DB ---
		console.log(`[ContextStore - _loadAndProcessContext] Context ${contextId} loaded from DB.`);
		// Convert StorableViewNodes to ViewNodes with Tweens, reusing old tweens if possible
		for (const [nodeId, storableNode] of storableContext.viewNodes) {
			const focalPlacement: AbsoluteTransform = { x: focalInitialStateFromOldContext.x, y: focalInitialStateFromOldContext.y, scale: focalInitialStateFromOldContext.scale };
			const targetState = _calculateTargetState(nodeId, contextId, focalPlacement, storableNode);
			const existingViewNode = oldContext?.viewNodes.get(nodeId);

			if (existingViewNode) {
				existingViewNode.state.set(targetState, NODE_TWEEN_OPTIONS); // Update existing tween
				finalViewNodes.set(nodeId, existingViewNode); // Reuse ViewNode object
			} else {
				finalViewNodes.set(nodeId, { id: nodeId, state: new Tween(targetState, NODE_TWEEN_OPTIONS) });
			}
		}
		// Convert stored viewport settings
		finalViewportSettings = _convertStorableViewportSettings(storableContext.viewportSettings, finalViewNodes.get(contextId));

	} else {
		// --- Context needs creation ---
		console.log(`[ContextStore - _loadAndProcessContext] Context ${contextId} not found in DB, creating new.`);
		contextWasCreated = true;
		// Use getNode from LocalAdapter directly, NodeStore might not be populated yet
		const focalDataNode = await localAdapter.getNode(contextId);
		if (!focalDataNode) throw new Error(`Cannot create context for non-existent node: ${contextId}`);

		// Create initial state and ViewNode for the focal node
		const correctedFocalInitialState: TweenableNodeState = {
			x: focalInitialStateFromOldContext.x,
			y: focalInitialStateFromOldContext.y,
			scale: focalInitialStateFromOldContext.scale,
			rotation: focalInitialStateFromOldContext.rotation,
			width: focalInitialStateFromOldContext.width,
			height: focalInitialStateFromOldContext.height
		};
		finalViewNodes.set(contextId, { id: contextId, state: new Tween(correctedFocalInitialState, { duration: 0 }) });
		// Default viewport for new context
		finalViewportSettings = { ...DEFAULT_VIEWPORT_SETTINGS }; // Use default, don't calculate based on window here
	}

	// --- Add Previous Focal Node (if context is new and applicable) ---
	if (contextWasCreated) {
		const currentHistory = get(historyStack);
		const previousContextId = currentHistory.length > 0 ? currentHistory[currentHistory.length - 1] : null;
		if (previousContextId && previousContextId !== contextId && !finalViewNodes.has(previousContextId)) {
			const previousFocalViewNode = oldContext?.viewNodes.get(previousContextId);
			if (previousFocalViewNode) {
				// Ensure the previous node's state is set correctly (might need tweening?)
				// For simplicity, just add it as is for now. It will tween on next switch *to* it.
				finalViewNodes.set(previousContextId, previousFocalViewNode);
			}
		}
	}

	// --- Add Default Connected Nodes (if context didn't just load them) ---
	const directlyConnectedEdges = await localAdapter.getEdgesByNodeIds([contextId]);
	const connectedNodeIdsToAdd = new Set<NodeId>();
	for (const edge of directlyConnectedEdges.values()) {
		const neighborId = edge.source === contextId ? edge.target : edge.source;
		if (neighborId !== contextId && !finalViewNodes.has(neighborId)) {
			connectedNodeIdsToAdd.add(neighborId);
		}
	}

	if (connectedNodeIdsToAdd.size > 0) {
		console.log(`[ContextStore - _loadAndProcessContext] Adding ${connectedNodeIdsToAdd.size} default connected nodes.`);
		contextWasCreated = true; // Mark modified if defaults added
		let defaultOffset = 150;
		const angleIncrement = 360 / connectedNodeIdsToAdd.size;
		let currentAngle = 0;
		const focalState = finalViewNodes.get(contextId)!.state.current;

		// Ensure DataNodes for connected nodes exist before getting defaults
		const connectedDataNodes = await localAdapter.getDataNodesByIds(Array.from(connectedNodeIdsToAdd));

		for (const connectedId of connectedNodeIdsToAdd) {
			const existingViewNodeInOldContext = oldContext?.viewNodes.get(connectedId);
			const connectedDataNode = connectedDataNodes.get(connectedId);
			const nodeType = connectedDataNode?.ntype ?? 'generic';
			const defaultConnectedState = getDefaultViewNodeStateForType(nodeType);

			// Calculate default position relative to current focal state
			const angleRad = (currentAngle * Math.PI) / 180;
			const defaultRelX = defaultOffset * Math.cos(angleRad);
			const defaultRelY = defaultOffset * Math.sin(angleRad);
			// Apply focal scale to relative offset
			const scaledRelX = defaultRelX; // Don't scale offset by focal scale initially
			const scaledRelY = defaultRelY;
			const defaultAbsX = focalState.x + scaledRelX;
			const defaultAbsY = focalState.y + scaledRelY;

			const defaultState: TweenableNodeState = {
				x: defaultAbsX,
				y: defaultAbsY,
				scale: 1, // Default nodes start at scale 1 relative to canvas
				rotation: defaultConnectedState.rotation,
				width: defaultConnectedState.width,
				height: defaultConnectedState.height
			};

			if (existingViewNodeInOldContext) {
				// If reusing from old context, update its state to the calculated default
				existingViewNodeInOldContext.state.set(defaultState, NODE_TWEEN_OPTIONS);
				finalViewNodes.set(connectedId, existingViewNodeInOldContext);
			} else {
				// Create new ViewNode/Tween
				finalViewNodes.set(connectedId, { id: connectedId, state: new Tween(defaultState, NODE_TWEEN_OPTIONS) });
			}
			currentAngle += angleIncrement;
		}
	}

	// Construct the final Context object
	const finalContext: Context = {
		id: contextId,
		viewNodes: finalViewNodes,
		viewportSettings: finalViewportSettings
	};

	// Save immediately if it was newly created or defaults were added
	if (contextWasCreated && localAdapter) {
		await localAdapter.saveContext(finalContext);
	}

	return { finalContext, wasCreated: contextWasCreated };
}

// Helper to calculate target state from storable node data
function _calculateTargetState(
	nodeId: NodeId,
	contextId: NodeId,
	focalPlacement: AbsoluteTransform, // This is the target state of the *focal* node
	storableNode: StorableViewNode
): TweenableNodeState {
	 if (nodeId === contextId) {
		// The focal node's state IS the focalPlacement (plus its own rotation/size)
		return {
			x: focalPlacement.x, y: focalPlacement.y,
			scale: focalPlacement.scale, rotation: storableNode.rotation,
			width: storableNode.width, height: storableNode.height
		};
	} else {
		// Calculate absolute position based on focal node's target state and stored relative offset
		// Note: Stored relX/relY are relative to the focal node's *origin* (center)
		// Note: Stored relScale is relative to the focal node's scale
		const absScale = focalPlacement.scale * storableNode.relScale;
		// Apply focal node's rotation *before* adding scaled relative offset? No, offset is likely in parent's frame.
		// Apply focal node's scale to the relative offset
		const scaledRelX = storableNode.relX * focalPlacement.scale;
		const scaledRelY = storableNode.relY * focalPlacement.scale;
		// TODO: Account for focal node rotation if relX/relY are defined within its rotated frame.
		// Assuming for now relX/relY are in the *parent's* coordinate frame relative to focal center.
		const absX = focalPlacement.x + scaledRelX;
		const absY = focalPlacement.y + scaledRelY;
		return {
			x: absX, y: absY, scale: absScale, rotation: storableNode.rotation,
			width: storableNode.width, height: storableNode.height
		};
	}
}

// Helper to convert storable viewport settings
function _convertStorableViewportSettings(
	storableSettings: StorableViewportSettings | undefined,
	focalViewNode: ViewNode | undefined // The focal ViewNode *in the target context*
): ViewportSettings | undefined {
	if (!storableSettings) return undefined;

	// The stored relPosX/Y represent the screen position of the focal node's origin (center)
	// We need to convert this back to the absolute canvas position (posX, posY) of the viewport's top-left corner.
	// posX = screen_focal_X - canvas_focal_X * scale
	// posY = screen_focal_Y - canvas_focal_Y * scale

	const focalState = focalViewNode?.state.current; // Use the target state
	if (focalState) {
		// Calculate the absolute canvas position (top-left) of the viewport
		const absPosX = storableSettings.relPosX - (focalState.x * storableSettings.scale);
		const absPosY = storableSettings.relPosY - (focalState.y * storableSettings.scale);
		return { scale: storableSettings.scale, posX: absPosX, posY: absPosY };
	} else {
		console.warn("[ContextStore - _convertStorableViewportSettings] Focal node state not found when converting viewport settings.");
		return { ...DEFAULT_VIEWPORT_SETTINGS }; // Fallback to default
	}
}

/**
 * Loads the necessary DataNodes and Edges for a given Context's ViewNodes.
 */
async function _loadContextData(context: Context | undefined): Promise<{ loadedDataNodes: Map<NodeId, DataNode>, loadedEdges: Map<EdgeId, KartaEdge> }> {
	let loadedDataNodes = new Map<NodeId, DataNode>();
	let loadedEdgesMap = new Map<EdgeId, KartaEdge>();

	if (!localAdapter || !context) {
		return { loadedDataNodes, loadedEdges: loadedEdgesMap };
	}

	const viewNodeIds = Array.from(context.viewNodes.keys());
	if (viewNodeIds.length > 0) {
		// Fetch DataNodes for all nodes visible in the context
		loadedDataNodes = await localAdapter.getDataNodesByIds(viewNodeIds);

		// Fetch all edges connected to any node in the view
		const allConnectedEdges = await localAdapter.getEdgesByNodeIds(viewNodeIds);

		// Filter edges to only include those where *both* source and target are in the current viewNodeIds
		const viewNodeIdSet = new Set(viewNodeIds);
		for (const [edgeId, edge] of allConnectedEdges.entries()) {
			if (viewNodeIdSet.has(edge.source) && viewNodeIdSet.has(edge.target)) {
				loadedEdgesMap.set(edgeId, edge);
			}
		}

	} else {
		console.warn(`[ContextStore - _loadContextData] Context ${context.id} has no view nodes.`);
	}
	return { loadedDataNodes, loadedEdges: loadedEdgesMap };
}


// --- Public Actions ---

/** Adds a new ViewNode to the current context */
export async function addViewNodeToCurrentContext(
	nodeId: NodeId,
	canvasX: number,
	canvasY: number,
	initialWidth?: number,
	initialHeight?: number
) {
	const dataNode = get(nodeStore).get(nodeId); // Get DataNode from NodeStore
	if (!dataNode) {
		console.error(`[ContextStore - addViewNodeToCurrentContext] DataNode ${nodeId} not found.`);
		return;
	}

	const contextId = get(currentContextId);
	const defaultViewState = getDefaultViewNodeStateForType(dataNode.ntype);
	const initialState: TweenableNodeState = {
		x: canvasX,
		y: canvasY,
		width: initialWidth ?? defaultViewState.width,
		height: initialHeight ?? defaultViewState.height,
		scale: defaultViewState.scale,
		rotation: defaultViewState.rotation
	};

	const newViewNode: ViewNode = {
		id: nodeId,
		state: new Tween(initialState, { duration: 0 }) // Initialize instantly
	};

	let saved = false;
	contexts.update(ctxMap => {
		let currentCtx = ctxMap.get(contextId);
		if (!currentCtx) {
			console.warn(`[ContextStore - addViewNodeToCurrentContext] Context ${contextId} not found. Creating context.`);
			// This scenario should ideally be handled by ensuring context exists before node creation
			// For safety, create a minimal context here.
			currentCtx = { id: contextId, viewNodes: new Map() };
			ctxMap.set(contextId, currentCtx);
		}
		currentCtx.viewNodes.set(nodeId, newViewNode);

		// Persist immediately after adding the view node
		if (localAdapter && currentCtx) {
			// Capture current viewport state when saving context
			currentCtx.viewportSettings = { ...viewTransform.current };
			localAdapter.saveContext(currentCtx)
				.then(() => {
					saved = true;
					console.log(`[ContextStore - addViewNodeToCurrentContext] Context ${contextId} saved after adding ViewNode ${nodeId}.`);
				})
				.catch(err => console.error(`[ContextStore - addViewNodeToCurrentContext] Error saving context:`, err));
		} else {
			console.warn("[ContextStore - addViewNodeToCurrentContext] LocalAdapter not available, context not saved.");
		}
		return ctxMap; // Return the modified map
	});

	if (!saved && localAdapter) {
		// If the update function didn't run or save failed initially
		console.warn("[ContextStore - addViewNodeToCurrentContext] Context save might not have completed synchronously.");
	}
}


// Update ViewNode Layout (Position, Scale, Rotation, Size)
export function updateViewNodeState(nodeId: NodeId, newState: Partial<TweenableNodeState>) {
	const contextId = get(currentContextId);

	contexts.update(ctxMap => {
		const currentCtx = ctxMap.get(contextId);
		const viewNode = currentCtx?.viewNodes.get(nodeId);

		if (viewNode) {
			// Create the target state by merging current state with new partial state
			const targetState: TweenableNodeState = {
				...viewNode.state.current, // Start with current values
				...newState // Overwrite with new values
			};

			// Update the tween instantly (no animation during direct manipulation like drag/resize)
			viewNode.state.set(targetState, { duration: 0 });

			// Persist the change asynchronously
			if (localAdapter && currentCtx) {
				// Debounce saving? For now, save on every update.
				currentCtx.viewportSettings = { ...viewTransform.current }; // Capture viewport
				localAdapter.saveContext(currentCtx)
					.catch(err => console.error(`[ContextStore - updateViewNodeState] Error saving context ${contextId}:`, err));
			}
			return ctxMap; // Return map to trigger updates if needed (though mutation might suffice)
		} else {
			console.warn(`[ContextStore - updateViewNodeState] ViewNode ${nodeId} not found in context ${contextId}.`);
			return ctxMap; // Return original map
		}
	});
}


// Context Switching
export async function switchContext(newContextId: NodeId, isUndoRedo: boolean = false) {
	const oldContextId = get(currentContextId);
	if (newContextId === oldContextId) return; // No change

	clearSelection(); // Clear selection from SelectionStore

	// --- History Management ---
	if (!isUndoRedo) {
		pushHistory(oldContextId); // Use action from HistoryStore
	}
	// --- End History Management ---

	if (!localAdapter) {
		console.error("[ContextStore - switchContext] LocalAdapter not available."); return;
	}
	console.log(`[ContextStore - switchContext] Switching from ${oldContextId} to ${newContextId}`);

	// --- Phase 1: Save Old Context State (Async) ---
	const oldContext = get(contexts).get(oldContextId);
	if (oldContext) {
		oldContext.viewportSettings = { ...viewTransform.current }; // Capture current viewport from ViewportStore
		localAdapter.saveContext(oldContext)
			.then(() => console.log(`[ContextStore - switchContext] Old context ${oldContextId} saved.`))
			.catch(error => console.error(`[ContextStore - switchContext] Error saving old context ${oldContextId}:`, error));
	} else {
		console.warn(`[ContextStore - switchContext] Old context ${oldContextId} not found in memory for saving.`);
	}

	// --- Phase 2: Load and Process New Context ---
	try {
		const focalInitialState = await _getFocalNodeInitialState(newContextId, oldContext);
		const { finalContext: processedContext } = await _loadAndProcessContext(
			newContextId,
			focalInitialState,
			oldContext
		);

		if (!processedContext) {
			throw new Error("Failed to load or create the new context object.");
		}

		// Load necessary DataNodes and Edges for the nodes now in the context
		const { loadedDataNodes, loadedEdges } = await _loadContextData(processedContext);

		// --- Phase 3: Update Svelte Stores ---
		// Update nodes and edges stores (Mark-and-Sweep)
		const currentNodesMap = get(nodeStore);
		const currentEdgesMap = get(edgeStore);
		const nextNodes = new Map<NodeId, DataNode>();
		const nextEdges = new Map<EdgeId, KartaEdge>(); // Use EdgeId type

		// Add/update nodes needed for the new context
		for (const [nodeId, dataNode] of loadedDataNodes.entries()) {
			nextNodes.set(nodeId, dataNode);
		}
		// Add/update edges relevant to the new context
		for (const [edgeId, edge] of loadedEdges.entries()) {
			// Edge loading logic already ensures both nodes are in viewNodeIds
			nextEdges.set(edgeId, edge);
		}
		// Update stores directly
		nodeStore.set(nextNodes);
		edgeStore.set(nextEdges);

		// Update contexts store with the fully processed context
		contexts.update(map => map.set(newContextId, processedContext));

		// Update current context ID
		currentContextId.set(newContextId);

		// --- Phase 4: Update Viewport ---
		const newViewportSettings = processedContext.viewportSettings;
		if (newViewportSettings !== undefined) {
			viewTransform.set(newViewportSettings, { duration: VIEWPORT_TWEEN_DURATION }); // Use tween from ViewportStore
		} else {
			console.log(`[ContextStore - switchContext] Context ${newContextId} was newly created or had no saved viewport; viewport position maintained.`);
			// Optionally set to default if desired
			// viewTransform.set(DEFAULT_VIEWPORT_SETTINGS, { duration: VIEWPORT_TWEEN_DURATION });
		}
		console.log(`[ContextStore - switchContext] Successfully switched to context ${newContextId}`);

	} catch (error) {
		console.error(`[ContextStore - switchContext] Error switching context to ${newContextId}:`, error);
		// Revert history if switch failed?
		if (!isUndoRedo) {
			historyStack.update(stack => stack.slice(0, -1)); // Pop the failed entry
			futureStack.set([]); // Ensure future is clear
		}
		// TODO: Consider reverting currentContextId or showing an error state
	}
}

// --- Context List Fetching ---
export async function fetchAvailableContextDetails(): Promise<{ id: NodeId, name: string, path: string }[]> {
	if (!localAdapter) {
		console.error("[ContextStore - fetchAvailableContextDetails] LocalAdapter not available.");
		return [];
	}
	try {
		// Get all context IDs, then fetch the corresponding DataNodes
		const contextIds = await localAdapter.getAllContextIds();
		if (contextIds.length === 0) {
			return [];
		}
		const dataNodesMap = await localAdapter.getDataNodesByIds(contextIds);

		// Define the type for the mapped object
		type ContextDetail = { id: NodeId, name: string, path: string };

		const contextDetails: ContextDetail[] = Array.from(dataNodesMap.values())
			.map((node: DataNode): ContextDetail => ({ // Add explicit types
				id: node.id,
				name: node.attributes?.name ?? `Node ${node.id.substring(0, 8)}`, // Fallback name
				path: node.path ?? `/${node.attributes?.name ?? node.id.substring(0, 8)}` // Fallback path
			}))
			.sort((a: ContextDetail, b: ContextDetail) => a.path.localeCompare(b.path)); // Add explicit types

		return contextDetails;

	} catch (error) {
		console.error("[ContextStore - fetchAvailableContextDetails] Error fetching context details:", error);
		return [];
	}
}

// --- Initialization Logic (Placeholder) ---
// Full initialization should likely live in a separate `initialize.ts`
// and call necessary setup functions in each store.
export function initializeContextStore() {
	// Subscribe to save last context ID
	currentContextId.subscribe(id => {
		if (typeof window !== 'undefined' && window.localStorage) {
			localStorage.setItem('karta-last-context-id', id);
			// console.log(`[ContextStore] Saved last context ID to localStorage: ${id}`);
		}
	});
	console.log("[ContextStore] Initialized subscriptions.");
}