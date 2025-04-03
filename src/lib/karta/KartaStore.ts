import { writable, get } from 'svelte/store';
import { Tween, tweened } from 'svelte/motion';
import { cubicOut } from 'svelte/easing';
import { localAdapter } from '../util/LocalAdapter';
import type { DataNode, KartaEdge, ViewNode, Context, Tool, NodeId, EdgeId, AbsoluteTransform, ViewportSettings } from '../types/types'; // Import ViewportSettings
import { MoveTool } from '../tools/MoveTool';
import { ConnectTool } from '../tools/ConnectTool';
import { ContextTool } from '../tools/ContextTool';
import { v4 as uuidv4 } from 'uuid';

// Define the Root Node ID
export const ROOT_NODE_ID = '00000000-0000-0000-0000-000000000000';

// Define default transform for root context or when focal node isn't visible
const DEFAULT_FOCAL_TRANSFORM: AbsoluteTransform = { x: 0, y: 0, scale: 1, rotation: 0 };
const DEFAULT_VIEWPORT_SETTINGS: ViewportSettings = { scale: 1, posX: 0, posY: 0 }; // Default viewport state
const VIEWPORT_TWEEN_DURATION = 500;

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

// --- Internal Helper Functions ---

/**
 * Determines the absolute transform for a target context's focal node.
 * Uses the existing position if visible in the old context, otherwise defaults.
 */
function _getFocalTransform(targetNodeId: NodeId, oldContextId: NodeId): AbsoluteTransform {
    const oldCtx = get(contexts).get(oldContextId);
    const targetViewNodeInOldCtx = oldCtx?.viewNodes.get(targetNodeId);

    if (targetViewNodeInOldCtx) {
        const transform = {
            x: targetViewNodeInOldCtx.x, y: targetViewNodeInOldCtx.y,
            scale: targetViewNodeInOldCtx.scale, rotation: targetViewNodeInOldCtx.rotation
        };
        return transform;
    } else {
        return { ...DEFAULT_FOCAL_TRANSFORM };
    }
}

/**
 * Loads a context from DB or creates it, adds default connected nodes (preserving existing positions), saves if modified.
 */
async function _loadAndUpdateContextWithDefaults(
    contextId: NodeId,
    focalAbsTransform: AbsoluteTransform,
    oldContextViewNodes: Map<NodeId, ViewNode> | undefined
): Promise<{ loadedContext: Context | undefined, contextModified: boolean }> {

    if (!localAdapter) return { loadedContext: undefined, contextModified: false };

    let loadedContext = await localAdapter.getContext(contextId, focalAbsTransform);
    let contextModified = false;


    if (!loadedContext) {
        console.log(`[_loadAndUpdateContextWithDefaults] Context ${contextId} not found in DB, creating new.`);
        const focalDataNode = await localAdapter.getNode(contextId);
        if (!focalDataNode) throw new Error(`Cannot create context for non-existent node: ${contextId}`);
        const focalViewNode: ViewNode = {
            id: contextId, x: focalAbsTransform.x, y: focalAbsTransform.y,
            width: 100, height: 100, scale: focalAbsTransform.scale, rotation: focalAbsTransform.rotation,
        };
        loadedContext = { id: contextId, viewNodes: new Map([[contextId, focalViewNode]]) }; // viewportSettings will be undefined initially
        contextModified = true; // Mark for saving as it was created
    }


    // Add Default Connected Nodes (runs every time context is switched to)
    const directlyConnectedEdges = await localAdapter.getEdgesByNodeIds([contextId]);
    const connectedNodeIdsToAdd = new Set<NodeId>();

    for (const edge of directlyConnectedEdges.values()) {
        const neighborId = edge.source === contextId ? edge.target : edge.source;
        if (neighborId !== contextId && !loadedContext.viewNodes.has(neighborId)) {
            connectedNodeIdsToAdd.add(neighborId);
        }
    }


    if (connectedNodeIdsToAdd.size > 0) {
        let defaultOffset = 150;
        const angleIncrement = 360 / connectedNodeIdsToAdd.size;
        let currentAngle = 0;

        for (const connectedId of connectedNodeIdsToAdd) {
            const existingViewNodeInOldContext = oldContextViewNodes?.get(connectedId);
            let defaultViewNode: ViewNode;

            if (existingViewNodeInOldContext) {
                 defaultViewNode = { ...existingViewNodeInOldContext };
            } else {
                // Calculate default position (simplified - no rotation applied)
                const angleRad = (currentAngle * Math.PI) / 180;
                const defaultRelX = defaultOffset * Math.cos(angleRad);
                const defaultRelY = defaultOffset * Math.sin(angleRad);
                const scaledRelX = defaultRelX * focalAbsTransform.scale;
                const scaledRelY = defaultRelY * focalAbsTransform.scale;
                const defaultAbsX = focalAbsTransform.x + scaledRelX;
                const defaultAbsY = focalAbsTransform.y + scaledRelY;

                defaultViewNode = {
                    id: connectedId, x: defaultAbsX, y: defaultAbsY,
                    width: 100, height: 100, scale: focalAbsTransform.scale, rotation: 0,
                };
                currentAngle += angleIncrement;
            }
            loadedContext.viewNodes.set(connectedId, defaultViewNode);
            contextModified = true;
        }
    } 


    // Save the context immediately if it was newly created or defaults were added
    if (contextModified) {
        await localAdapter.saveContext(loadedContext);
    }

    return { loadedContext, contextModified };
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
    loadedContext: Context,
    loadedDataNodes: Map<NodeId, DataNode>,
    loadedEdges: Map<EdgeId, KartaEdge>
) {
    const currentNodes = get(nodes);
    const currentEdges = get(edges);
    const nodesToRemove = new Set(currentNodes.keys());
    const edgesToRemove = new Set(currentEdges.keys());
    const newNodes = new Map<NodeId, DataNode>();
    const newEdges = new Map<EdgeId, KartaEdge>();

    // Process loaded nodes
    for (const [nodeId, dataNode] of loadedDataNodes.entries()) {
        newNodes.set(nodeId, dataNode);
        nodesToRemove.delete(nodeId);
    }

    // Process loaded edges
    for (const [edgeId, edge] of loadedEdges.entries()) {
        if (newNodes.has(edge.source) && newNodes.has(edge.target)) {
            newEdges.set(edgeId, edge);
            edgesToRemove.delete(edgeId);
        } else {
            // console.warn(`[_applyStoresUpdate] Edge ${edgeId} skipped: Source or target node not in new context.`);
        }
    }

    // Set the new state
    nodes.set(newNodes);
    edges.set(newEdges);
    contexts.update(map => map.set(newContextId, loadedContext)); // Update the specific context's view data
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
    if (localAdapter) {
        while (await localAdapter.checkNameExists(finalName)) {
            finalName = `${baseName}${counter}`; counter++;
        }
    } else { console.warn("LocalAdapter not ready, cannot check for duplicate names."); }

	const newNodeData: DataNode = {
		id: newNodeId, ntype: ntype, createdAt: now, modifiedAt: now,
        path: `/${finalName}`,
		attributes: { ...attributes, name: finalName },
	};
    const newViewNode: ViewNode = {
        id: newNodeId, x: canvasX, y: canvasY,
        width: 100, height: 100, scale: 1, rotation: 0,
    };
    const contextId = get(currentContextId);
	nodes.update(n => n.set(newNodeId, newNodeData));
    contexts.update(ctxMap => {
        let currentCtx = ctxMap.get(contextId);
        if (!currentCtx) { currentCtx = { id: contextId, viewNodes: new Map() }; }
        currentCtx.viewNodes.set(newNodeId, newViewNode);
        ctxMap.set(contextId, currentCtx);
        return ctxMap;
    });
    if (localAdapter) {
        try {
            await localAdapter.saveNode(newNodeData);
            const updatedCtx = get(contexts).get(contextId);
            if (updatedCtx) {
                // Capture current viewport settings when saving context after node creation
                updatedCtx.viewportSettings = viewTransform.target;
                await localAdapter.saveContext(updatedCtx);
            }
            console.log(`[KartaStore] Node ${newNodeId} and Context ${contextId} saved.`);
        } catch (error) { console.error("Error saving node or context after creation:", error); }
    } else { console.warn("LocalAdapter not initialized, persistence disabled."); }
}

// Node Layout Update
export function updateNodeLayout(nodeId: NodeId, newX: number, newY: number) {
    const contextId = get(currentContextId);
    contexts.update(ctxMap => {
        const currentCtx = ctxMap.get(contextId);
        if (currentCtx?.viewNodes.has(nodeId)) {
            const viewNode = currentCtx.viewNodes.get(nodeId)!;
            const updatedViewNode = { ...viewNode, x: newX, y: newY };
            currentCtx.viewNodes.set(nodeId, updatedViewNode);
            ctxMap.set(contextId, currentCtx);
            if (localAdapter) {
                // Capture current viewport settings when saving context after layout update
                currentCtx.viewportSettings = viewTransform.target;
                localAdapter.saveContext(currentCtx).catch(err => console.error(`Error saving context ${contextId} after layout update:`, err));
            }
        } else { console.warn(`ViewNode ${nodeId} not found in context ${contextId} for layout update`); }
        return ctxMap;
    });
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
    const sourceViewNode = context?.viewNodes.get(nodeId);
    if (sourceViewNode) {
        isConnecting.set(true);
        connectionSourceNodeId.set(nodeId);
        const startX = sourceViewNode.x + sourceViewNode.width / 2;
        const startY = sourceViewNode.y + sourceViewNode.height / 2;
        tempLineTargetPosition.set({ x: startX, y: startY });
        console.log("Starting connection from:", nodeId);
    } else { console.warn(`Cannot start connection: ViewNode ${nodeId} not found in current context ${contextId}`); }
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
    if (newContextId === oldContextId) {
        return;
    }
    if (!localAdapter) {
        console.error("[switchContext] LocalAdapter not available."); return;
    }
    // Save Old Context (Async)
    const oldContext = get(contexts).get(oldContextId);
    const oldFocalTransform = _getFocalTransform(oldContextId, oldContextId);

    // Capture current viewport settings before saving old context
    // Create a COPY and convert absolute to relative coordinates for saving
    let viewportSettingsToSave = { ...viewTransform.target }; // Create a shallow copy
    viewportSettingsToSave.posX = (viewportSettingsToSave.posX + oldFocalTransform.x);
    viewportSettingsToSave.posY = (viewportSettingsToSave.posY + oldFocalTransform.y);

    console.log("--------------NEW CYCLE----------------")

    console.log("Old focal transform", oldFocalTransform);
    console.log("Absolute viewport position", viewTransform.target);
    console.log("OLD RELATIVE Viewport settings to save", viewportSettingsToSave);

    if (oldContext) {
        oldContext.viewportSettings = viewportSettingsToSave; // Save the modified copy
        localAdapter.saveContext(oldContext)
            .catch(error => console.error(`[switchContext] Error saving old context ${oldContextId} (async):`, error));
    } else { console.warn(`[switchContext] Old context ${oldContextId} not found in memory store.`); }

    try {
        // Determine Focal Transform
        const newFocalAbsTransform = _getFocalTransform(newContextId, oldContextId);

        // Load/Create Context, Add Defaults, Save if needed
        const { loadedContext } = await _loadAndUpdateContextWithDefaults(
            newContextId,
            newFocalAbsTransform,
            oldContext?.viewNodes // Pass previous view nodes for position preservation
        );

        if (!loadedContext) {
            throw new Error("Failed to load or create the new context object.");
        }

        // Load required DataNodes and Edges
        const { loadedDataNodes, loadedEdges } = await _loadContextData(loadedContext);

        // Apply Mark-and-Sweep Update to Stores
        _applyStoresUpdate(newContextId, loadedContext, loadedDataNodes, loadedEdges);

        // Apply Viewport Tween
        let newSettings = loadedContext.viewportSettings;
        if (newSettings) {
            console.log("new focal transform", newFocalAbsTransform);
            console.log("LOADED RELATIVE: ", newSettings);
            newSettings.posX = newSettings.posX - newFocalAbsTransform.x;
            newSettings.posY = newSettings.posY - newFocalAbsTransform.y;

            console.log("NEW ABSOLUTE", newSettings);

            viewTransform.set(newSettings, { duration: VIEWPORT_TWEEN_DURATION }); // Use tween
        }

    } catch (error) {
        console.error(`[switchContext] Error switching context to ${newContextId}:`, error);
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
async function _loadInitialRootContext(rootDataNode: DataNode): Promise<Context> {
     if (!localAdapter) throw new Error("LocalAdapter not available for initialization.");
     const rootContextId = ROOT_NODE_ID;
     let rootContextModified = false;
     let loadedRootContext = await localAdapter.getContext(rootContextId, DEFAULT_FOCAL_TRANSFORM);
     let finalRootContext: Context;

     if (loadedRootContext) {
         console.log("Root context loaded from DB.");
         finalRootContext = loadedRootContext;
     } else {
         console.log("Root context not found in DB, creating in memory...");
         finalRootContext = { id: rootContextId, viewNodes: new Map() };
         rootContextModified = true;
     }

     if (!finalRootContext.viewNodes.has(rootContextId)) {
         console.log("Root ViewNode missing from context, adding default...");
         const rootViewNode: ViewNode = {
             id: rootContextId, x: DEFAULT_FOCAL_TRANSFORM.x, y: DEFAULT_FOCAL_TRANSFORM.y,
             width: 150, height: 100, scale: DEFAULT_FOCAL_TRANSFORM.scale, rotation: DEFAULT_FOCAL_TRANSFORM.rotation,
         };
         finalRootContext.viewNodes.set(rootContextId, rootViewNode);
         rootContextModified = true;
     }

     // Add default connected nodes check during initialization as well
     console.log(`[_loadInitialRootContext] Checking for default connected nodes for Root...`);
     const directlyConnectedEdges = await localAdapter.getEdgesByNodeIds([rootContextId]);
     const connectedNodeIdsToAdd = new Set<NodeId>();
     for (const edge of directlyConnectedEdges.values()) {
         const neighborId = edge.source === rootContextId ? edge.target : edge.source;
         if (neighborId !== rootContextId && !finalRootContext.viewNodes.has(neighborId)) {
             connectedNodeIdsToAdd.add(neighborId);
         }
     }
     if (connectedNodeIdsToAdd.size > 0) {
         console.log(`[_loadInitialRootContext] Found ${connectedNodeIdsToAdd.size} connected nodes to add by default to Root:`, Array.from(connectedNodeIdsToAdd));
         let defaultOffset = 150;
         const angleIncrement = 360 / connectedNodeIdsToAdd.size;
         let currentAngle = 0;
         for (const connectedId of connectedNodeIdsToAdd) {
             // Calculate default position (simplified)
             const angleRad = (currentAngle * Math.PI) / 180;
             const defaultRelX = defaultOffset * Math.cos(angleRad);
             const defaultRelY = defaultOffset * Math.sin(angleRad);
             const scaledRelX = defaultRelX * DEFAULT_FOCAL_TRANSFORM.scale; // Root scale is 1
             const scaledRelY = defaultRelY * DEFAULT_FOCAL_TRANSFORM.scale;
             const defaultAbsX = DEFAULT_FOCAL_TRANSFORM.x + scaledRelX; // Root X is 0
             const defaultAbsY = DEFAULT_FOCAL_TRANSFORM.y + scaledRelY; // Root Y is 0
             const defaultViewNode: ViewNode = {
                 id: connectedId, x: defaultAbsX, y: defaultAbsY,
                 width: 100, height: 100, scale: DEFAULT_FOCAL_TRANSFORM.scale, rotation: 0,
             };
             finalRootContext.viewNodes.set(connectedId, defaultViewNode);
             currentAngle += angleIncrement;
             rootContextModified = true;
         }
     }

     // Ensure viewportSettings exist before saving
     if (!finalRootContext.viewportSettings) {
         finalRootContext.viewportSettings = { ...DEFAULT_VIEWPORT_SETTINGS };
         rootContextModified = true; // Need to save if we added default settings
     }

     if (rootContextModified) {
         await localAdapter.saveContext(finalRootContext);
     }
     return finalRootContext;
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
            const finalRootContext = await _loadInitialRootContext(rootDataNode);
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
