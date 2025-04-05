import { writable, get } from 'svelte/store';
import { Tween, tweened } from 'svelte/motion';
import { cubicOut } from 'svelte/easing';
import { localAdapter } from '../util/LocalAdapter';
import type { DataNode, KartaEdge, ViewNode, Context, Tool, NodeId, EdgeId, AbsoluteTransform, ViewportSettings, TweenableNodeState, StorableContext, StorableViewNode, StorableViewportSettings } from '../types/types'; // Added StorableViewportSettings
import { MoveTool } from '../tools/MoveTool';
import { ConnectTool } from '../tools/ConnectTool';
import { ContextTool } from '../tools/ContextTool';
import { v4 as uuidv4 } from 'uuid';

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

        // Create initial state and ViewNode for the focal node
        const focalInitialState: TweenableNodeState = {
            x: focalTargetPlacement.x, y: focalTargetPlacement.y,
            scale: focalTargetPlacement.scale, rotation: 0, // Default rotation
            width: 100, height: 100 // Default size - TODO: Get from DataNode attributes?
        };
        finalViewNodes.set(contextId, { id: contextId, state: new Tween(focalInitialState, { duration: 0 }) });
        finalViewportSettings = { ...DEFAULT_VIEWPORT_SETTINGS };
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
                // Reuse old ViewNode/Tween but set to default state
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
        viewportSettings: finalViewportSettings ?? { ...DEFAULT_VIEWPORT_SETTINGS } // Ensure settings exist
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

    // 2. Create initial state for the ViewNode's tween
    const initialState: TweenableNodeState = {
        x: canvasX, y: canvasY,
        width: 100, height: 100, // TODO: Get default size based on ntype?
        scale: 1, rotation: 0
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


export async function switchContext(newContextId: NodeId) {
    const oldContextId = get(currentContextId);
    if (newContextId === oldContextId) return; // No change
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
        const newViewportSettings = processedContext.viewportSettings;
        if (newViewportSettings) {
            viewTransform.set(newViewportSettings, { duration: VIEWPORT_TWEEN_DURATION });
        } else {
             viewTransform.set({ ...DEFAULT_VIEWPORT_SETTINGS }, { duration: VIEWPORT_TWEEN_DURATION });
             console.warn(`[switchContext] No viewport settings found for context ${newContextId}, resetting.`);
        }
        console.log(`[switchContext] Successfully switched to context ${newContextId}`);

    } catch (error) {
        console.error(`[switchContext] Error switching context to ${newContextId}:`, error);
        // Consider reverting to oldContextId or showing an error state
    }
}


// --- Initialization (Refactored) ---

/** Ensures the Root DataNode exists in the database. */
async function _ensureRootDataNode(): Promise<DataNode> {
    if (!localAdapter) throw new Error("LocalAdapter not available for initialization.");
    let rootDataNode = await localAdapter.getNode(ROOT_NODE_ID);
    if (!rootDataNode) {
        console.log("Root DataNode not found in DB, creating...");
        const now = Date.now();
        rootDataNode = {
            id: ROOT_NODE_ID, ntype: 'root', createdAt: now, modifiedAt: now,
            path: '/', attributes: { name: 'Root' },
        };
        await localAdapter.saveNode(rootDataNode);
    } else {
        console.log("Root DataNode found in DB.");
    }
    return rootDataNode;
}

/** Loads or creates the initial Root Context. */
// Renamed from _loadInitialRootContext to reflect it returns the processed Context
async function _loadAndProcessInitialRootContext(rootDataNode: DataNode): Promise<Context> {
    if (!localAdapter) throw new Error("LocalAdapter not available for initialization.");
    const rootContextId = ROOT_NODE_ID;

    // Use the main context processing function, passing undefined for oldContext
    const { finalContext, wasCreated } = await _loadAndProcessContext(
        rootContextId,
        DEFAULT_FOCAL_TRANSFORM, // Root's target placement is the default
        undefined // No old context during initialization
    );

    if (!finalContext) {
        throw new Error("Failed to load or create the initial root context.");
    }

    // Ensure viewport settings exist (might be redundant if _loadAndProcessContext handles it)
     if (!finalContext.viewportSettings) {
         finalContext.viewportSettings = { ...DEFAULT_VIEWPORT_SETTINGS };
         // If settings were missing, we should probably save
         if (!wasCreated) { // Avoid double save if context was just created
             await localAdapter.saveContext(finalContext);
         }
     }


    return finalContext;
}

/** Loads initial DataNodes and Edges for the Root Context. */
async function _loadInitialRootData(rootContext: Context, rootDataNode: DataNode): Promise<{ dataNodesForRoot: Map<NodeId, DataNode>, edgesForRoot: Map<EdgeId, KartaEdge> }> {
    if (!localAdapter) return { dataNodesForRoot: new Map(), edgesForRoot: new Map() };

    const viewNodeIds = Array.from(rootContext.viewNodes.keys());
    let dataNodesForRoot = new Map<NodeId, DataNode>();
    let edgesForRoot = new Map<EdgeId, KartaEdge>();

    if (viewNodeIds.length > 0) {
        dataNodesForRoot = await localAdapter.getDataNodesByIds(viewNodeIds);

        edgesForRoot = await localAdapter.getEdgesByNodeIds(viewNodeIds);
    }

    // Ensure Root DataNode is included
    if (!dataNodesForRoot.has(ROOT_NODE_ID)) {
         console.warn("Root DataNode was missing from loaded nodes, adding it back.");
         dataNodesForRoot.set(ROOT_NODE_ID, rootDataNode);
    }
    return { dataNodesForRoot, edgesForRoot };
}


async function initializeStores() {
    get(currentTool)?.activate(); // Activate default tool

    if (localAdapter) {
        try {
            const rootDataNode = await _ensureRootDataNode();
            const finalRootContext = await _loadAndProcessInitialRootContext(rootDataNode);
            const { dataNodesForRoot, edgesForRoot } = await _loadInitialRootData(finalRootContext, rootDataNode);

            // Set initial store state
            nodes.set(dataNodesForRoot);
            edges.set(edgesForRoot);
            contexts.set(new Map([[ROOT_NODE_ID, finalRootContext]]));
            currentContextId.set(ROOT_NODE_ID);

            // Set initial viewport state (no tween)
            const initialSettings = finalRootContext.viewportSettings || DEFAULT_VIEWPORT_SETTINGS;
            viewTransform.set(initialSettings, { duration: 0 });

        } catch (error) {
            console.error("Error during store initialization:", error);
            // Set default empty state on error
            nodes.set(new Map());
            edges.set(new Map());
            contexts.set(new Map());
            currentContextId.set(ROOT_NODE_ID);
            viewTransform.set({ ...DEFAULT_VIEWPORT_SETTINGS }, { duration: 0 }); // Reset viewport on error
        }
    } else {
        console.warn("LocalAdapter not initialized, skipping initialization in SSR.");
        // Set default empty state for SSR
        nodes.set(new Map());
        edges.set(new Map());
        contexts.set(new Map());
        currentContextId.set(ROOT_NODE_ID);
        viewTransform.set({ ...DEFAULT_VIEWPORT_SETTINGS }, { duration: 0 }); // Set default viewport for SSR
    }
}

// Run initialization only in browser environment
if (typeof window !== 'undefined' ) {
    initializeStores();
}

