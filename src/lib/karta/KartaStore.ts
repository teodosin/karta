import { writable, get } from 'svelte/store';
import { Tween } from 'svelte/motion'; // Removed 'tweened' as it's not used directly here
import { cubicOut } from 'svelte/easing';
import { localAdapter } from '../util/LocalAdapter';
import type { DataNode, KartaEdge, ViewNode, Context, Tool, NodeId, EdgeId, AbsoluteTransform, ViewportSettings, TweenableNodeState, StorableContext, StorableViewNode, StorableViewportSettings } from '../types/types';
import { MoveTool } from '../tools/MoveTool';
import { ConnectTool } from '../tools/ConnectTool';
import { ContextTool } from '../tools/ContextTool';
import { v4 as uuidv4 } from 'uuid';
import { getDefaultViewNodeStateForType, getDefaultAttributesForType } from '$lib/node_types/registry'; // Import the new helpers

// Define the Root Node ID
export const ROOT_NODE_ID = '00000000-0000-0000-0000-000000000000';

// Define default transform for root context or when focal node isn't visible
const DEFAULT_FOCAL_TRANSFORM: AbsoluteTransform = { x: 0, y: 0, scale: 1 }; // Represents target placement x, y, scale
const DEFAULT_VIEWPORT_SETTINGS: ViewportSettings = { scale: 1, posX: 0, posY: 0 }; // Default viewport state
const VIEWPORT_TWEEN_DURATION = 500;
const NODE_TWEEN_DURATION = 250; // Duration for node transitions
const NODE_TWEEN_OPTIONS = { duration: NODE_TWEEN_DURATION, easing: cubicOut };

// --- Store Definition ---
export const viewTransform = new Tween<ViewportSettings>( // Use ViewportSettings type
	{ ...DEFAULT_VIEWPORT_SETTINGS }, // Initialize with default
	{ duration: VIEWPORT_TWEEN_DURATION, easing: cubicOut } // Default tween settings
);
export const nodes = writable<Map<NodeId, DataNode>>(new Map());
export const edges = writable<Map<EdgeId, KartaEdge>>(new Map());
export const contexts = writable<Map<NodeId, Context>>(new Map());
export const currentContextId = writable<NodeId>(ROOT_NODE_ID);
export const currentTool = writable<Tool>(new MoveTool());
export const isConnecting = writable<boolean>(false);
export const connectionSourceNodeId = writable<NodeId | null>(null);
export const tempLineTargetPosition = writable<{ x: number; y: number } | null>(null);
// Removed currentTransformTweens store
export const selectedNodeIds = writable<Set<NodeId>>(new Set());

// --- Create Node Menu Stores ---
export const isCreateNodeMenuOpen = writable<boolean>(false);
export const createNodeMenuPosition = writable<{ screenX: number; screenY: number; canvasX: number; canvasY: number } | null>(null);

// --- History Stores ---
export const historyStack = writable<NodeId[]>([]);
export const futureStack = writable<NodeId[]>([]);

// --- Internal Helper Functions ---

/**
 * Determines the absolute transform for a target context's focal node.
 * Uses the existing position if visible in the old context, otherwise defaults.
 */
function _getFocalPlacement(targetNodeId: NodeId, oldContext: Context | undefined): AbsoluteTransform {
    // Renamed for clarity: determines target x, y, scale for the focal node
    const targetViewNodeInOldCtx = oldContext?.viewNodes.get(targetNodeId);

    if (targetViewNodeInOldCtx) {
        // Get the current state from the tween
        const currentState = targetViewNodeInOldCtx.state.current;
        // Return only the properties matching AbsoluteTransform
        return {
            x: currentState.x,
            y: currentState.y,
            scale: currentState.scale
        };
    } else {
        // Return default placement (x, y, scale only)
        return { ...DEFAULT_FOCAL_TRANSFORM };
    }
}

/**
 * Loads a context from DB or creates it, adds default connected nodes (preserving existing positions), saves if modified.
 */
async function _loadAndProcessContext(
    contextId: NodeId,
    focalTargetPlacement: AbsoluteTransform, // Target x, y, scale for the focal node
    oldContext: Context | undefined // Previous context for potential tween reuse
): Promise<{ finalContext: Context | undefined, wasCreated: boolean }> {

    if (!localAdapter) return { finalContext: undefined, wasCreated: false };

    const storableContext = await localAdapter.getContext(contextId);
    let contextWasCreated = false;
    const finalViewNodes = new Map<NodeId, ViewNode>();
    let finalViewportSettings: ViewportSettings | undefined = undefined;

    if (storableContext) {
        // --- Context exists in DB ---
        console.log(`[_loadAndProcessContext] Context ${contextId} loaded from DB.`);
        // Convert StorableViewNodes to ViewNodes with Tweens, reusing old tweens if possible
        for (const [nodeId, storableNode] of storableContext.viewNodes) {
            const targetState = _calculateTargetState(nodeId, contextId, focalTargetPlacement, storableNode);
            const existingViewNode = oldContext?.viewNodes.get(nodeId);

            if (existingViewNode) {
                existingViewNode.state.set(targetState, NODE_TWEEN_OPTIONS); // Update existing tween
                finalViewNodes.set(nodeId, existingViewNode); // Reuse ViewNode object
            } else {
                // Create new ViewNode with a new Tween, starting from target state? Or current if exists? Start from target.
                finalViewNodes.set(nodeId, { id: nodeId, state: new Tween(targetState, NODE_TWEEN_OPTIONS) });
            }
        }
        // Convert stored viewport settings
        finalViewportSettings = _convertStorableViewportSettings(storableContext.viewportSettings, finalViewNodes.get(contextId));

    } else {
        // --- Context needs creation ---
        console.log(`[_loadAndProcessContext] Context ${contextId} not found in DB, creating new.`);
        contextWasCreated = true;
        const focalDataNode = await localAdapter.getNode(contextId);
        if (!focalDataNode) throw new Error(`Cannot create context for non-existent node: ${contextId}`);
        if (contextId === ROOT_NODE_ID) {
             console.log(`[_loadAndProcessContext] Root context creation: Loaded focalDataNode ntype: ${focalDataNode.ntype}`);
        }

        // Create initial state and ViewNode for the focal node
        // Use type-specific defaults for size/scale/rotation
        if (contextId === ROOT_NODE_ID) {
             console.log(`[_loadAndProcessContext] Root context creation: Getting default view state for ntype: ${focalDataNode.ntype}`);
        }
        const defaultViewState = getDefaultViewNodeStateForType(focalDataNode.ntype);
        const focalInitialState: TweenableNodeState = {
            x: focalTargetPlacement.x, y: focalTargetPlacement.y,
            // Use defaults from registry, but override scale with placement scale if needed?
            // For now, let's prioritize the type default scale.
            ...defaultViewState, // Includes width, height, scale, rotation
            scale: defaultViewState.scale * focalTargetPlacement.scale // Apply placement scale relative to default? Or just use placement scale? Let's use placement scale directly for consistency.
            // Re-evaluate scale logic if needed. Let's use placement scale directly.
            // scale: focalTargetPlacement.scale, // Use placement scale
            // rotation: defaultViewState.rotation // Use default rotation
        };
        // Corrected state using placement scale directly:
        const correctedFocalInitialState: TweenableNodeState = {
             x: focalTargetPlacement.x, y: focalTargetPlacement.y,
             scale: focalTargetPlacement.scale, // Use placement scale
             rotation: defaultViewState.rotation, // Use default rotation from type
             width: defaultViewState.width, // Use default width from type
             height: defaultViewState.height // Use default height from type
        };
        finalViewNodes.set(contextId, { id: contextId, state: new Tween(correctedFocalInitialState, { duration: 0 }) });
        // For newly created contexts, don't set viewport settings yet.
        // Let the viewport remain where it is. Settings will be saved on first interaction/switch away.
        finalViewportSettings = {
            scale: 1,
            posX: window.innerWidth / 2,
            posY: window.innerHeight / 2
        };
    }

    // --- Add Default Connected Nodes (if context didn't just load them) ---
    // This ensures nodes connected *after* the context was last saved are still added by default
    const directlyConnectedEdges = await localAdapter.getEdgesByNodeIds([contextId]);
    const connectedNodeIdsToAdd = new Set<NodeId>();
    for (const edge of directlyConnectedEdges.values()) {
        const neighborId = edge.source === contextId ? edge.target : edge.source;
        if (neighborId !== contextId && !finalViewNodes.has(neighborId)) {
            connectedNodeIdsToAdd.add(neighborId);
        }
    }

    if (connectedNodeIdsToAdd.size > 0) {
        console.log(`[_loadAndProcessContext] Adding ${connectedNodeIdsToAdd.size} default connected nodes.`);
        contextWasCreated = true; // Mark modified if defaults added
        let defaultOffset = 150;
        const angleIncrement = 360 / connectedNodeIdsToAdd.size;
        let currentAngle = 0;
        const focalState = finalViewNodes.get(contextId)!.state.current; // Focal state must exist

        for (const connectedId of connectedNodeIdsToAdd) {
            const existingViewNodeInOldContext = oldContext?.viewNodes.get(connectedId);
            // Calculate default position relative to current focal state
            const angleRad = (currentAngle * Math.PI) / 180;
            const defaultRelX = defaultOffset * Math.cos(angleRad);
            const defaultRelY = defaultOffset * Math.sin(angleRad);
            const scaledRelX = defaultRelX * focalState.scale;
            const scaledRelY = defaultRelY * focalState.scale;
            const defaultAbsX = focalState.x + scaledRelX;
            const defaultAbsY = focalState.y + scaledRelY;
            const defaultState: TweenableNodeState = {
                x: defaultAbsX, y: defaultAbsY, scale: focalState.scale, rotation: 0,
                width: 100, height: 100 // Default size - TODO: Get from DataNode?
            };

            if (existingViewNodeInOldContext) {
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
        viewportSettings: finalViewportSettings // Allow undefined if context was new/had no settings
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
    focalPlacement: AbsoluteTransform,
    storableNode: StorableViewNode
): TweenableNodeState {
     if (nodeId === contextId) {
        return {
            x: focalPlacement.x, y: focalPlacement.y,
            scale: focalPlacement.scale, rotation: storableNode.rotation,
            width: storableNode.width, height: storableNode.height
        };
    } else {
        const absScale = focalPlacement.scale * storableNode.relScale;
        const scaledRelX = storableNode.relX * focalPlacement.scale;
        const scaledRelY = storableNode.relY * focalPlacement.scale;
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
    focalViewNode: ViewNode | undefined
): ViewportSettings | undefined {
    if (!storableSettings) return undefined;

    const focalState = focalViewNode?.state.current;
    if (focalState) {
        const absPosX = storableSettings.relPosX - (focalState.x * storableSettings.scale);
        const absPosY = storableSettings.relPosY - (focalState.y * storableSettings.scale);
        return { scale: storableSettings.scale, posX: absPosX, posY: absPosY };
    } else {
        console.warn("Focal node state not found when converting viewport settings.");
        return { ...DEFAULT_VIEWPORT_SETTINGS };
    }
}

/**
 * Loads the necessary DataNodes and Edges for a given Context.
 */
async function _loadContextData(context: Context | undefined): Promise<{ loadedDataNodes: Map<NodeId, DataNode>, loadedEdges: Map<EdgeId, KartaEdge> }> {
    let loadedDataNodes = new Map<NodeId, DataNode>();
    let loadedEdges = new Map<EdgeId, KartaEdge>();

    if (!localAdapter || !context) {
        return { loadedDataNodes, loadedEdges };
    }

    const viewNodeIds = Array.from(context.viewNodes.keys());
    if (viewNodeIds.length > 0) {
        loadedDataNodes = await localAdapter.getDataNodesByIds(viewNodeIds);

        loadedEdges = await localAdapter.getEdgesByNodeIds(viewNodeIds);
    } else {
        console.warn(`[_loadContextData] Context ${context.id} has no view nodes.`);
    }
    return { loadedDataNodes, loadedEdges };
}

/**
 * Applies the mark-and-sweep update to the main stores.
 */
function _applyStoresUpdate(
    newContextId: NodeId,
    processedContext: Context, // The fully processed context with ViewNodes/Tweens
    loadedDataNodes: Map<NodeId, DataNode>,
    loadedEdges: Map<EdgeId, KartaEdge>
) {
    // --- Update nodes and edges stores (Mark-and-Sweep) ---
    const currentNodesMap = get(nodes);
    const currentEdgesMap = get(edges);
    const nodesToRemove = new Set(currentNodesMap.keys());
    const edgesToRemove = new Set(currentEdgesMap.keys());
    const nextNodes = new Map<NodeId, DataNode>();
    const nextEdges = new Map<EdgeId, KartaEdge>();

    // Add/update nodes needed for the new context
    for (const [nodeId, dataNode] of loadedDataNodes.entries()) {
        nextNodes.set(nodeId, dataNode);
        nodesToRemove.delete(nodeId); // Mark this node to be kept
    }
    // Add/update edges relevant to the new context
    for (const [edgeId, edge] of loadedEdges.entries()) {
        // Only keep edges where both source and target nodes are present in the new context
        if (processedContext.viewNodes.has(edge.source) && processedContext.viewNodes.has(edge.target)) {
             nextEdges.set(edgeId, edge);
             edgesToRemove.delete(edgeId); // Mark this edge to be kept
        }
    }
    // Update stores
    nodes.set(nextNodes);
    edges.set(nextEdges);

    // --- Update contexts store ---
    // Set the fully processed context (containing ViewNodes with Tweens)
    contexts.update(map => map.set(newContextId, processedContext));

    // --- Update current context ID ---
    currentContextId.set(newContextId);
}


// --- Public Store Actions ---

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
    } else if (viewportEl) { // Added check for viewportEl for error message context
        console.error('[KartaStore] Cannot create node from menu: Position not found in store.');

        closeCreateNodeMenu(); // Close menu after creation
    } else {
        console.error('[KartaStore] Cannot create node from menu: Viewport element not found.');
        closeCreateNodeMenu(); // Close menu even if creation failed
    }
}



// Tool Management
const toolInstances = { move: new MoveTool(), connect: new ConnectTool(), context: new ContextTool() };
export function setTool(toolName: 'move' | 'connect' | 'context') {
    const current = get(currentTool);
    const next = toolInstances[toolName];
    if (current !== next) {
        current?.deactivate(); next?.activate(); currentTool.set(next);
        console.log(`Switched tool to: ${toolName}`);
    }
}

// --- Selection Actions ---

/** Clears the current selection. */
export function clearSelection() {
selectedNodeIds.update(currentSelection => {
    if (currentSelection.size > 0) {
        console.log('[Selection] Cleared');
        return new Set<NodeId>(); // Return new Set to trigger update
    }
    return currentSelection; // No change if already empty
});
}

/**
* Sets the selection to the provided node IDs.
* @param nodeIds A single NodeId or an array/Set of NodeIds.
*/
export function setSelectedNodes(nodeIds: NodeId | NodeId[] | Set<NodeId>) {
const idsToSelect = new Set(Array.isArray(nodeIds) ? nodeIds : nodeIds instanceof Set ? Array.from(nodeIds) : [nodeIds]);
selectedNodeIds.set(idsToSelect);
console.log('[Selection] Set:', Array.from(idsToSelect));
}


/** Deselects a specific node. */
export function deselectNode(nodeId: NodeId) {
selectedNodeIds.update(currentSelection => {
const deleted = currentSelection.delete(nodeId);
if (deleted) {
console.log('[Selection] Deselected:', nodeId, ' New selection:', Array.from(currentSelection));
return new Set(currentSelection); // Return new Set to trigger update
}
return currentSelection; // No change
});
}

/** Toggles the selection state of a single node. */
export function toggleSelection(nodeId: NodeId) {
selectedNodeIds.update(currentSelection => {
    if (currentSelection.has(nodeId)) {
        currentSelection.delete(nodeId);
        console.log('[Selection] Toggled OFF:', nodeId, ' New selection:', Array.from(currentSelection));
    } else {
        currentSelection.add(nodeId);
        console.log('[Selection] Toggled ON:', nodeId, ' New selection:', Array.from(currentSelection));
    }
    return new Set(currentSelection); // Return new Set to trigger update
});
}

// Node Creation
export async function createNodeAtPosition(canvasX: number, canvasY: number, ntype: string = 'text', attributes: Record<string, any> = {}) {
	const newNodeId: NodeId = uuidv4();
    const now = Date.now();
    let baseName = attributes.name || ntype;
    let finalName = baseName;
    let counter = 2;
    // Ensure name uniqueness (assuming checkNameExists works correctly)
    if (localAdapter) {
        while (await localAdapter.checkNameExists(finalName)) {
            finalName = `${baseName}${counter}`; counter++;
        }
    } else { console.warn("LocalAdapter not ready, cannot check for duplicate names."); }

	// 1. Create DataNode
	const newNodeData: DataNode = {
		id: newNodeId, ntype: ntype, createdAt: now, modifiedAt: now,
        path: `/${finalName}`, // Simple path for now
		attributes: { ...attributes, name: finalName },
	};

    // 2. Get default view state based on ntype and create initial state for the ViewNode's tween
    const defaultViewState = getDefaultViewNodeStateForType(ntype);
    const initialState: TweenableNodeState = {
        x: canvasX, y: canvasY,
        ...defaultViewState // Use width, height, scale, rotation from registry helper
    };

    // 3. Create the new ViewNode containing the Tween
    const newViewNode: ViewNode = {
        id: newNodeId,
        state: new Tween(initialState, { duration: 0 }) // Initialize instantly, no animation
    };

    // 4. Update stores
    const contextId = get(currentContextId);
	nodes.update(n => n.set(newNodeId, newNodeData)); // Add DataNode

    contexts.update(ctxMap => {
        let currentCtx = ctxMap.get(contextId);
        if (!currentCtx) {
            console.warn(`Context ${contextId} not found when creating node ${newNodeId}. Creating context.`);
            currentCtx = { id: contextId, viewNodes: new Map() };
            ctxMap.set(contextId, currentCtx); // Add new context to map if needed
        }
        currentCtx.viewNodes.set(newNodeId, newViewNode); // Add new ViewNode to context
        return ctxMap; // Return the modified map to trigger update
    });

    // 5. Persist changes
    if (localAdapter) {
        try {
            await localAdapter.saveNode(newNodeData);
            const updatedCtx = get(contexts).get(contextId);
            if (updatedCtx) {
                updatedCtx.viewportSettings = { ...viewTransform.current }; // Capture viewport state
                await localAdapter.saveContext(updatedCtx); // Save context
            }
            console.log(`[KartaStore] Node ${newNodeId} and Context ${contextId} saved.`);
        } catch (error) { console.error("Error saving node or context after creation:", error); }
    } else { console.warn("LocalAdapter not initialized, persistence disabled."); }
}

// Node Layout Update
export function updateNodeLayout(nodeId: NodeId, newX: number, newY: number) {
    const contextId = get(currentContextId);
    const currentCtx = get(contexts).get(contextId);
    const viewNode = currentCtx?.viewNodes.get(nodeId); // This is ViewNode { id, state: Tween }

    if (viewNode) {
        // 1. Get the current state from the tween
        const currentState = viewNode.state.current;

        // 2. Create the new target state with updated position
        const newState: TweenableNodeState = {
            ...currentState, // Preserve scale, rotation, width, height
            x: newX,
            y: newY
        };

        // 3. Update the tween instantly (no animation during drag)
        viewNode.state.set(newState, { duration: 0 });

        // 4. Trigger reactivity for the contexts store
        // Although we mutated the tween *within* the ViewNode, explicitly
        // signalling that the contexts map has changed ensures Svelte updates.
        contexts.update(map => map);

        // 5. Persist the change asynchronously
        // saveContext reads the .current state from the tween when converting
        if (localAdapter && currentCtx) {
            // Debounce saving? For now, save on every update during drag.
            currentCtx.viewportSettings = { ...viewTransform.current }; // Capture viewport
            localAdapter.saveContext(currentCtx)
                .catch(err => console.error(`Error saving context ${contextId} during layout update:`, err));
        }
    } else {
        console.warn(`ViewNode ${nodeId} not found in context ${contextId} for layout update`);
    }
}
// Node Attribute Update
export async function updateNodeAttributes(nodeId: NodeId, newAttributes: Record<string, any>) {
    const currentNodes = get(nodes);
    const dataNode = currentNodes.get(nodeId);

    if (!dataNode) {
        console.warn(`[updateNodeAttributes] DataNode ${nodeId} not found in store.`);
        return;
    }

    // Prevent renaming system nodes
    if (dataNode.attributes?.isSystemNode) {
        console.warn(`[updateNodeAttributes] Attempted to modify attributes of system node ${nodeId}. Operation cancelled.`);
        return;
    }

    const oldName = dataNode.attributes?.name;
    const newName = newAttributes?.name;
    let attributesToSave = { ...dataNode.attributes, ...newAttributes }; // Merge old and new

    // Check for name change and uniqueness
    if (newName && newName.trim() && newName !== oldName) {
        const finalNewName = newName.trim();
        if (localAdapter) {
            const nameExists = await localAdapter.checkNameExists(finalNewName);
            if (nameExists) {
                console.error(`[updateNodeAttributes] Cannot rename node ${nodeId} to "${finalNewName}". Name already exists.`);
                // TODO: Add user feedback (e.g., toast notification)
                return; // Prevent update if name exists
            }
            // Update attributesToSave only if name is valid and unique
            attributesToSave.name = finalNewName;
        } else {
            console.warn("[updateNodeAttributes] LocalAdapter not ready, cannot check for duplicate names.");
            // Proceed with name change but without uniqueness check? Or prevent? Let's prevent for safety.
            console.error("[updateNodeAttributes] Cannot verify name uniqueness. Rename cancelled.");
            return;
        }
    } else if (newName !== undefined && !newName.trim()) {
        console.warn(`[updateNodeAttributes] Attempted to rename node ${nodeId} to an empty name. Operation cancelled.`);
        return; // Prevent empty names
    }

    // Create updated node data
    // Create updated node data
    const updatedNodeData: DataNode = {
        ...dataNode,
        attributes: attributesToSave, // Use potentially updated attributesToSave
        modifiedAt: Date.now(),
        // Potentially update path if name changed? For now, keep path separate.
        // path: `/${attributesToSave.name}` // Example if path should sync
    };

    // Update the store
    // Update the store only if changes were actually made (or if it's not just a name change)
    // This prevents unnecessary updates if only whitespace was trimmed from name
    if (JSON.stringify(updatedNodeData.attributes) !== JSON.stringify(dataNode.attributes)) {
        nodes.update(n => n.set(nodeId, updatedNodeData));
        console.log(`[updateNodeAttributes] Updated attributes for node ${nodeId}:`, attributesToSave);

        // Persist changes
        if (localAdapter) {
            try {
                await localAdapter.saveNode(updatedNodeData);
            } catch (error) {
                console.error(`[updateNodeAttributes] Error saving node ${nodeId} after attribute update:`, error);
                // Optionally revert store update on save failure?
                // nodes.update(n => n.set(nodeId, dataNode)); // Example revert
            }
        } else {
            console.warn("[updateNodeAttributes] LocalAdapter not initialized, persistence disabled.");
        }
    } else {
        console.log(`[updateNodeAttributes] No effective attribute changes for node ${nodeId}.`);
    }
}


// Edge Creation
export async function createEdge(sourceId: NodeId, targetId: NodeId) {
    if (sourceId === targetId) { console.warn("Cannot connect node to itself."); return; }
    const currentEdges = get(edges);
    for (const edge of currentEdges.values()) {
        if ((edge.source === sourceId && edge.target === targetId) || (edge.source === targetId && edge.target === sourceId)) {
            console.warn(`Edge between ${sourceId} and ${targetId} already exists.`); return;
        }
    }
    const newEdgeId: EdgeId = uuidv4();
	const newEdge: KartaEdge = { id: newEdgeId, source: sourceId, target: targetId };
	edges.update(e => e.set(newEdgeId, newEdge));
	console.log(`Created edge ${newEdgeId} between ${sourceId} and ${targetId}`);
    if (localAdapter) {
        try { await localAdapter.saveEdge(newEdge); }
        catch (error) { console.error("Error saving edge:", error); }
    } else { console.warn("LocalAdapter not initialized, persistence disabled."); }
}

// Connection Process Helpers
export function startConnectionProcess(nodeId: NodeId) {
    if (get(currentTool)?.name !== 'connect' || get(isConnecting)) return;
    const contextId = get(currentContextId);
    const context = get(contexts).get(contextId);
    const sourceViewNode = context?.viewNodes.get(nodeId); // This is ViewNode { id, state: Tween }
    if (sourceViewNode) {
        isConnecting.set(true);
        connectionSourceNodeId.set(nodeId);
        // Access position/dimensions via the tween's current state
        const sourceState = sourceViewNode.state.current;
        const startX = sourceState.x + sourceState.width / 2;
        const startY = sourceState.y + sourceState.height / 2;
        tempLineTargetPosition.set({ x: startX, y: startY });
        console.log("Starting connection from:", nodeId);
    } else {
        console.warn(`Cannot start connection: ViewNode ${nodeId} not found in current context ${contextId}`);
    }
}
export function updateTempLinePosition(canvasX: number, canvasY: number) { if (get(isConnecting)) tempLineTargetPosition.set({x: canvasX, y: canvasY}); }
export function finishConnectionProcess(targetNodeId: NodeId | null) {
    if (!get(isConnecting)) return;
    const sourceId = get(connectionSourceNodeId);
    if (sourceId && targetNodeId) createEdge(sourceId, targetNodeId);
    else console.log("Connection cancelled.");
    isConnecting.set(false); connectionSourceNodeId.set(null); tempLineTargetPosition.set(null);
}
export function cancelConnectionProcess() { finishConnectionProcess(null); }

// Screen Coordinates Helper
export function screenToCanvasCoordinates(screenX: number, screenY: number, containerRect: DOMRect): { x: number; y: number } {
    const currentTransform = viewTransform.target;
	const canvasX = (screenX - containerRect.left - currentTransform.posX) / currentTransform.scale;
    const canvasY = (screenY - containerRect.top - currentTransform.posY) / currentTransform.scale;
	return { x: canvasX, y: canvasY };
}


export async function switchContext(newContextId: NodeId, isUndoRedo: boolean = false) { // Added isUndoRedo flag
    const oldContextId = get(currentContextId);
    if (newContextId === oldContextId) return; // No change

    // --- History Management ---
    if (!isUndoRedo) {
        historyStack.update(stack => [...stack, oldContextId]);
        futureStack.set([]); // Clear future stack on new action
        console.log(`[switchContext] Pushed ${oldContextId} to history.`);
    }
    // --- End History Management ---


    if (!localAdapter) {
        console.error("[switchContext] LocalAdapter not available."); return;
    }
    console.log(`[switchContext] Switching from ${oldContextId} to ${newContextId}`);

    // --- Phase 1: Save Old Context State (Async) ---
    const oldContext = get(contexts).get(oldContextId);
    if (oldContext) {
        oldContext.viewportSettings = { ...viewTransform.current }; // Capture current viewport
        localAdapter.saveContext(oldContext) // Adapter converts ViewNode with Tween back to Storable
            .then(() => console.log(`[switchContext] Old context ${oldContextId} saved.`))
            .catch(error => console.error(`[switchContext] Error saving old context ${oldContextId}:`, error));
    } else {
        console.warn(`[switchContext] Old context ${oldContextId} not found in memory for saving.`);
    }

    // --- Phase 2: Load and Process New Context ---
    try {
        // Determine target placement (x,y,scale) for the new focal node based on old context
        const focalTargetPlacement = _getFocalPlacement(newContextId, oldContext);

        // Load StorableContext, convert/merge into in-memory Context with Tweens, add defaults
        const { finalContext: processedContext } = await _loadAndProcessContext(
            newContextId,
            focalTargetPlacement,
            oldContext // Pass old context to reuse tweens
        );

        if (!processedContext) {
            throw new Error("Failed to load or create the new context object.");
        }

        // Load necessary DataNodes and Edges for the nodes now in the context
        const { loadedDataNodes, loadedEdges } = await _loadContextData(processedContext);

        // --- Phase 3: Update Svelte Stores ---
        _applyStoresUpdate(newContextId, processedContext, loadedDataNodes, loadedEdges);

        // --- Phase 4: Update Viewport ---
        // --- Phase 4: Update Viewport ---
        const newViewportSettings = processedContext.viewportSettings;
        // Only tween the viewport if settings were loaded for the context.
        // If undefined (meaning context was newly created), keep viewport as is.
        if (newViewportSettings !== undefined) {
            viewTransform.set(newViewportSettings, { duration: VIEWPORT_TWEEN_DURATION });
        } else {
            console.log(`[switchContext] Context ${newContextId} was newly created or had no saved viewport; viewport position maintained.`);
        }
        console.log(`[switchContext] Successfully switched to context ${newContextId}`);

    } catch (error) {
        console.error(`[switchContext] Error switching context to ${newContextId}:`, error);
        // Consider reverting to oldContextId or showing an error state
    }
}

// --- History Actions ---
export function undoContextSwitch() {
    const history = get(historyStack);
    if (history.length === 0) {
        console.log("[undoContextSwitch] History stack is empty.");
        return;
    }

    const previousId = history[history.length - 1]; // Get last element
    const currentId = get(currentContextId);

    historyStack.update(stack => stack.slice(0, -1)); // Remove last element
    futureStack.update(stack => [...stack, currentId]); // Add current to future

    console.log(`[undoContextSwitch] Undoing to: ${previousId}`);
    switchContext(previousId, true); // Call switchContext with isUndoRedo flag
}

export function redoContextSwitch() {
    const future = get(futureStack);
    if (future.length === 0) {
        console.log("[redoContextSwitch] Future stack is empty.");
        return;
    }

    const nextId = future[future.length - 1]; // Get last element
    const currentId = get(currentContextId);

    futureStack.update(stack => stack.slice(0, -1)); // Remove last element
    historyStack.update(stack => [...stack, currentId]); // Add current to history

    console.log(`[redoContextSwitch] Redoing to: ${nextId}`);
    switchContext(nextId, true); // Call switchContext with isUndoRedo flag
}


// --- Initialization (Refactored) ---

/** Generalized function to ensure a DataNode exists */
async function _ensureDataNodeExists(nodeId: NodeId): Promise<DataNode | null> {
    if (!localAdapter) {
        console.error(`[_ensureDataNodeExists] LocalAdapter not available while checking for ${nodeId}`);
        return null;
    }
    try {
        let dataNode = await localAdapter.getNode(nodeId);
        if (nodeId === ROOT_NODE_ID) {
            console.log(`[_ensureDataNodeExists] Root node check: Found in DB?`, !!dataNode, dataNode ? `Existing ntype: ${dataNode.ntype}` : '');
        }
        if (!dataNode) {
            console.warn(`[_ensureDataNodeExists] DataNode ${nodeId} not found. Creating default.`);
            const now = Date.now();
            // Determine default properties based on whether it's the root node
            const isRoot = nodeId === ROOT_NODE_ID;
            const defaultName = isRoot ? 'root' : `node-${nodeId.substring(0, 8)}`;
            const defaultPath = isRoot ? '/root' : `/${defaultName}`;
            const defaultNtype = isRoot ? 'root' : 'generic';
            if (isRoot) {
                console.log(`[_ensureDataNodeExists] Root node creation: Assigning ntype: ${defaultNtype}`);
            }

            dataNode = {
                id: nodeId,
                ntype: defaultNtype, // Set type correctly
                createdAt: now,
                modifiedAt: now,
                path: defaultPath, // Set path correctly
                attributes: { name: defaultName, ...(isRoot && { isSystemNode: true }) }, // Set name and system flag for root
            };
            await localAdapter.saveNode(dataNode);
            console.log(`[_ensureDataNodeExists] Default DataNode ${nodeId} created and saved.`);
        }
        return dataNode;
    } catch (error) {
        console.error(`[_ensureDataNodeExists] Error ensuring DataNode ${nodeId} exists:`, error);
        return null;
    }
}

// --- Context List Fetching ---
export async function fetchAvailableContextDetails(): Promise<{ id: NodeId, name: string, path: string }[]> {
    if (!localAdapter) {
        console.error("[fetchAvailableContextDetails] LocalAdapter not available.");
        return [];
    }
    try {
        const contextIds = await localAdapter.getAllContextIds();
        if (contextIds.length === 0) {
            return [];
        }

        const dataNodesMap = await localAdapter.getDataNodesByIds(contextIds);
        const contextDetails = Array.from(dataNodesMap.values())
            .map(node => ({
                id: node.id,
                name: node.attributes?.name ?? `Node ${node.id.substring(0, 8)}`, // Fallback name
                path: node.path ?? `/${node.attributes?.name ?? node.id.substring(0, 8)}` // Fallback path
            }))
            .sort((a, b) => a.path.localeCompare(b.path)); // Sort alphabetically by path

        return contextDetails;

    } catch (error) {
        console.error("[fetchAvailableContextDetails] Error fetching context details:", error);
        return [];
    }
}

// Removed _loadAndProcessInitialRootContext and _loadInitialRootData as they are replaced by the generalized functions

async function initializeStores() {
    console.log("[initializeStores] Initializing Karta stores...");
    get(currentTool)?.activate(); // Activate default tool

    if (!localAdapter) {
        console.error("[initializeStores] LocalAdapter not initialized. Cannot proceed.");
        // Set default empty state on critical error
        nodes.set(new Map());
        edges.set(new Map());
        contexts.set(new Map());
        currentContextId.set(ROOT_NODE_ID); // Default to root ID even on error
        viewTransform.set(DEFAULT_VIEWPORT_SETTINGS, { duration: 0 });
        setTool('move');
        return;
    }

    let targetInitialContextId = ROOT_NODE_ID; // Default
    let initialDataNode: DataNode | null = null;
    let initialProcessedContext: Context | undefined = undefined;
    let loadedDataNodes = new Map<NodeId, DataNode>();
    let loadedEdges = new Map<EdgeId, KartaEdge>();

    try {
        // 1. Determine Initial Context ID - FORCING ROOT FOR NOW
        // if (typeof window !== 'undefined' && window.localStorage) {
        //     const savedId = localStorage.getItem('karta-last-context-id');
        //     if (savedId) {
        //         targetInitialContextId = savedId;
        //         console.log(`[initializeStores] Found last context ID in localStorage: ${targetInitialContextId}`);
        //     } else {
        //         console.log(`[initializeStores] No last context ID found, defaulting to ROOT: ${ROOT_NODE_ID}`);
        //     }
        // } else {
        //     console.log(`[initializeStores] localStorage not available, defaulting to ROOT: ${ROOT_NODE_ID}`);
        // }
        targetInitialContextId = ROOT_NODE_ID; // FORCE START WITH ROOT
        console.log(`[initializeStores] Forcing start with ROOT_NODE_ID: ${targetInitialContextId}`);

        // 2. Ensure Initial DataNode Exists (Attempt target ID first)
        initialDataNode = await _ensureDataNodeExists(targetInitialContextId);

        // 3. Fallback to Root if target ID failed
        if (!initialDataNode) {
            console.warn(`[initializeStores] Failed to ensure target node ${targetInitialContextId} exists. Falling back to ROOT.`);
            targetInitialContextId = ROOT_NODE_ID; // Reset target ID to root
            initialDataNode = await _ensureDataNodeExists(ROOT_NODE_ID);
            if (!initialDataNode) {
                // If even the root fails, throw a critical error
                throw new Error("CRITICAL: Failed to ensure even the Root DataNode exists during initialization.");
            }
        }
        // 3.5 Ensure Root Node has isSystemNode flag (for backward compatibility)
        if (initialDataNode.id === ROOT_NODE_ID && !initialDataNode.attributes?.isSystemNode) {
            console.warn(`[initializeStores] Root node ${ROOT_NODE_ID} is missing the isSystemNode flag. Adding and saving...`);
            initialDataNode.attributes = { ...initialDataNode.attributes, isSystemNode: true };
            initialDataNode.modifiedAt = Date.now();
            try {
                await localAdapter.saveNode(initialDataNode);
                console.log(`[initializeStores] Successfully added isSystemNode flag to root node.`);
            } catch (saveError) {
                console.error(`[initializeStores] CRITICAL: Failed to save isSystemNode flag to root node:`, saveError);
                // Continue initialization, but the root might remain unprotected if saving failed.
            }
        }


        // 4. Load Initial Context & Data (Generalized)
        const { finalContext } = await _loadAndProcessContext(
            initialDataNode.id, // Use the validated ID
            DEFAULT_FOCAL_TRANSFORM,
            undefined // No old context on init
        );
        initialProcessedContext = finalContext;

        if (!initialProcessedContext) {
            throw new Error(`Failed to load or process initial context for node: ${initialDataNode.id}`);
        }

        const loadedData = await _loadContextData(initialProcessedContext);
        loadedDataNodes = loadedData.loadedDataNodes;
        loadedEdges = loadedData.loadedEdges;

        // 5. Apply Initial State
        _applyStoresUpdate(initialDataNode.id, initialProcessedContext, loadedDataNodes, loadedEdges);

        // 6. Set Initial Viewport
        const initialViewportSettings = initialProcessedContext.viewportSettings || DEFAULT_VIEWPORT_SETTINGS;
        viewTransform.set(initialViewportSettings, { duration: 0 }); // Set instantly

        console.log(`[initializeStores] Stores initialized successfully, starting context: ${initialDataNode.id}`);

    } catch (error) {
        console.error("[initializeStores] Error during store initialization:", error);
        // Set default empty state on error
        nodes.set(new Map());
        edges.set(new Map());
        contexts.set(new Map());
        currentContextId.set(ROOT_NODE_ID); // Default to root ID on error
        viewTransform.set(DEFAULT_VIEWPORT_SETTINGS, { duration: 0 });
    }

    // 7. Set Initial Tool (runs even if init fails partially)
    setTool('move');
}

// Run initialization only in browser environment
if (typeof window !== 'undefined' ) {
    initializeStores();
    
    // Subscribe to save last context ID
    currentContextId.subscribe(id => {
        if (typeof window !== 'undefined' && window.localStorage) { // Check if localStorage is available
            localStorage.setItem('karta-last-context-id', id);
            console.log(`[KartaStore] Saved last context ID to localStorage: ${id}`);
        }
    });
}

