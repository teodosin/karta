import { writable, get } from 'svelte/store';
import { tweened } from 'svelte/motion';
import { cubicOut } from 'svelte/easing';
import { localAdapter } from '../util/LocalAdapter';
// Import the new types from types.ts
import type { DataNode, KartaEdge, ViewNode, Context, Tool, NodeId, EdgeId, AbsoluteTransform } from '../types/types'; // Import AbsoluteTransform
import { MoveTool } from '../tools/MoveTool';
import { ConnectTool } from '../tools/ConnectTool';
import { ContextTool } from '../tools/ContextTool';
import { v4 as uuidv4 } from 'uuid'; // Import uuid for ID generation

// Define the Root Node ID
export const ROOT_NODE_ID = '00000000-0000-0000-0000-000000000000';

// --- Store Definition ---
// Removed local/redundant type definitions (DataNode, Edge, ViewNodeLayout, ContextId)

// Viewport State
export const viewTransform = tweened(
	{ scale: 1, posX: 0, posY: 0 },
	{ duration: 200, easing: cubicOut }
);

// Data State
export const nodes = writable<Map<NodeId, DataNode>>(new Map()); // Holds core DataNode info
export const edges = writable<Map<EdgeId, KartaEdge>>(new Map()); // Holds KartaEdge info
export const contexts = writable<Map<NodeId, Context>>(new Map()); // Holds Contexts (keyed by focal NodeId)
export const currentContextId = writable<NodeId>(ROOT_NODE_ID); // Start in the Root Node's context

// Interaction State
export const currentTool = writable<Tool>(new MoveTool()); // Default to MoveTool
export const isConnecting = writable<boolean>(false);
export const connectionSourceNodeId = writable<NodeId | null>(null);
export const tempLineTargetPosition = writable<{ x: number; y: number } | null>(null);


// --- Store Actions ---

// Tool Management
const toolInstances = {
    move: new MoveTool(),
    connect: new ConnectTool(),
    context: new ContextTool()
};

export function setTool(toolName: 'move' | 'connect' | 'context') {
    const current = get(currentTool);
    const next = toolInstances[toolName];
    if (current !== next) {
        current?.deactivate(); // Deactivate previous tool
        next?.activate(); // Activate new tool
        currentTool.set(next);
        console.log(`Switched tool to: ${toolName}`);
    }
}

export async function createNodeAtPosition(canvasX: number, canvasY: number, ntype: string = 'text', attributes: Record<string, any> = {}) {
	const newNodeId: NodeId = uuidv4();
    const now = Date.now();
	const newNodeData: DataNode = {
		id: newNodeId,
		ntype: ntype,
        createdAt: now,
        modifiedAt: now,
        path: `/node_${newNodeId}`, // Added: Simple default path
		attributes: {
            name: ntype, // Changed: Default name is now just the ntype
            ...attributes // Merge provided attributes
        },
	};

    // Create the ViewNode for the current context
    const newViewNode: ViewNode = {
        id: newNodeId,
        x: canvasX,
        y: canvasY,
        width: 100, // Default width
        height: 100, // Default height
        scale: 1,
        rotation: 0,
    };

    const contextId = get(currentContextId);

	// Update nodes store
	nodes.update(n => n.set(newNodeId, newNodeData));

    // Update contexts store
    contexts.update(ctxMap => {
        let currentCtx = ctxMap.get(contextId);
        if (!currentCtx) {
            // Create context if it doesn't exist
            currentCtx = { id: contextId, viewNodes: new Map() };
        }
        currentCtx.viewNodes.set(newNodeId, newViewNode);
        ctxMap.set(contextId, currentCtx);
        return ctxMap; // Return the modified map
    });


	console.log(`[KartaStore] createNodeAtPosition: Creating node ${newNodeId} (${newNodeData.attributes.name}) in context ${contextId}`);

    // Persist the DataNode and the updated Context
    if (localAdapter) {
        console.log(`[KartaStore] createNodeAtPosition: Attempting to save node ${newNodeId}...`);
        try {
            await localAdapter.saveNode(newNodeData); // Save DataNode
            console.log(`[KartaStore] createNodeAtPosition: Node ${newNodeId} saved.`);

            // Get the updated context to save its ViewNode layout
            const updatedCtx = get(contexts).get(contextId);
            // Get the focal transform for the current context
            // TODO: This assumes the focal transform is readily available.
            // For root context, it's ROOT_TRANSFORM. For others, it needs calculation.

            if (updatedCtx) {
                 console.log(`[KartaStore] createNodeAtPosition: Attempting to save context ${contextId}...`);
                await localAdapter.saveContext(updatedCtx); // Pass focal transform
                console.log(`[KartaStore] createNodeAtPosition: Context ${contextId} saved after node creation.`);
            } else {
                 console.error(`[KartaStore] createNodeAtPosition: Could not find context ${contextId} to persist after node creation.`);
            }

        } catch (error) {
            console.error("Error saving node or context after creation:", error);
        }
    } else {
        console.warn("LocalAdapter not initialized, persistence disabled in SSR.");
    }
    // TODO: Add persistence for Context/ViewNode updates later // <-- Still relevant for other context changes
}

export function updateNodeLayout(nodeId: NodeId, newX: number, newY: number) {
    const contextId = get(currentContextId);
    contexts.update(ctxMap => {
        const currentCtx = ctxMap.get(contextId);
        if (currentCtx) {
            const viewNode = currentCtx.viewNodes.get(nodeId);
            if (viewNode) {
                // Update the specific ViewNode's position
                const updatedViewNode = { ...viewNode, x: newX, y: newY };
                currentCtx.viewNodes.set(nodeId, updatedViewNode);
                ctxMap.set(contextId, currentCtx); // Put the updated context back

                // Persist the updated context - Re-enabled, will be configurable later
                if (localAdapter) {
                    localAdapter.saveContext(currentCtx).catch(err => {
                        console.error(`Error saving context ${contextId} after layout update:`, err);
                    });
                }
            } else {
                console.warn(`ViewNode ${nodeId} not found in context ${contextId}`);
            }
        } else {
            console.warn(`Context ${contextId} not found for layout update`);
        }
        return ctxMap; // Return the modified map
    });
    // Do NOT save DataNode here, layout changes don't affect core data
    // Consider updating DataNode's modifiedAt timestamp if needed, but maybe not for layout.
}


export async function createEdge(sourceId: NodeId, targetId: NodeId) {
    if (sourceId === targetId) {
        console.warn("Cannot connect node to itself.");
        return;
    }

    const currentEdges = get(edges);
    for (const edge of currentEdges.values()) {
        if ((edge.source === sourceId && edge.target === targetId) || (edge.source === targetId && edge.target === sourceId)) {
            console.warn(`Edge between ${sourceId} and ${targetId} already exists.`);
            return;
        }
    }

    const newEdgeId: EdgeId = uuidv4();
	const newEdge: KartaEdge = { // Use KartaEdge type
		id: newEdgeId,
		source: sourceId,
		target: targetId,
	};

	edges.update(e => e.set(newEdgeId, newEdge));
	console.log(`Created edge ${newEdgeId} between ${sourceId} and ${targetId}`);
    if (localAdapter) {
        try {
            await localAdapter.saveEdge(newEdge);
        } catch (error) {
            console.error("Error saving edge:", error);
        }
    } else {
        console.warn("LocalAdapter not initialized, persistence disabled in SSR.");
    }
}


export function startConnectionProcess(nodeId: NodeId) {
    if (get(currentTool)?.name !== 'connect') return; // Check tool name
    if (get(isConnecting)) return;

    const contextId = get(currentContextId);
    const context = get(contexts).get(contextId);
    const sourceViewNode = context?.viewNodes.get(nodeId);

    if (sourceViewNode) {
        isConnecting.set(true);
        connectionSourceNodeId.set(nodeId);
        // Start temp line from the center of the node
        const startX = sourceViewNode.x + sourceViewNode.width / 2;
        const startY = sourceViewNode.y + sourceViewNode.height / 2;
        tempLineTargetPosition.set({ x: startX, y: startY });
        console.log("Starting connection from:", nodeId);
    } else {
        console.warn(`Cannot start connection: ViewNode ${nodeId} not found in current context ${contextId}`);
    }
}

export function updateTempLinePosition(canvasX: number, canvasY: number) {
    if (!get(isConnecting)) return;
    tempLineTargetPosition.set({x: canvasX, y: canvasY});
}

export function finishConnectionProcess(targetNodeId: NodeId | null) {
    if (!get(isConnecting)) return;
    const sourceId = get(connectionSourceNodeId);

    if (sourceId && targetNodeId) {
        createEdge(sourceId, targetNodeId);
    } else {
        console.log("Connection cancelled.");
    }

    isConnecting.set(false);
    connectionSourceNodeId.set(null);
    tempLineTargetPosition.set(null);
}

export function cancelConnectionProcess() {
    finishConnectionProcess(null);
}

// --- Helper Functions ---

export function screenToCanvasCoordinates(
    screenX: number,
    screenY: number,
    containerRect: DOMRect
): { x: number; y: number } {
    const currentTransform = get(viewTransform);
	const canvasX = (screenX - containerRect.left - currentTransform.posX) / currentTransform.scale;
    const canvasY = (screenY - containerRect.top - currentTransform.posY) / currentTransform.scale;
	return { x: canvasX, y: canvasY };
}

// --- Context Switching ---

export async function switchContext(newContextId: NodeId) {

    const oldContextId = get(currentContextId);
    if (newContextId === oldContextId) {
        return; // Already in the target context
    }

    // 1. Save the old context (converting absolute to relative)
    const oldContext = get(contexts).get(oldContextId);
    if (oldContext && localAdapter) {
        try {
            await localAdapter.saveContext(oldContext);
        } catch (error) {
            console.error(`Error saving old context ${oldContextId}:`, error);
            // Decide if we should abort the switch here? For now, continue.
        }
    }

    // 2. Load the new context (converting relative to absolute)
    let newContext: Context | undefined;
    if (localAdapter) {
        try {
            newContext = await localAdapter.getContext(newContextId);
            if (newContext) {
                console.log(`Loaded context ${newContextId} from storage.`);
            } else {
                 console.log(`Context ${newContextId} not found in storage, creating new.`);
                 // Create new context if it doesn't exist
                 newContext = { id: newContextId, viewNodes: new Map() };
                 // TODO: Implement default population logic here
                 // If new focal exists in previous context, it is used as the new focal viewnode
                 const newFocal = 0
                 // Find nodes connected to newContextId in oldContext, add their ViewNodes.
                 // Save the newly created context immediately
                 await localAdapter.saveContext(newContext);
            }
        } catch (error) {
             console.error(`Error loading or creating context ${newContextId}:`, error);
             return; // Abort switch if loading/creation fails
        }
    } else {
        // No persistence - check if context exists in memory, create if not
        newContext = get(contexts).get(newContextId);
        if (!newContext) {
            newContext = { id: newContextId, viewNodes: new Map() };
             // TODO: Implement default population logic here too
        }
    }

    // 3. Update stores
    if (newContext) {
        // Update the main contexts map
        contexts.update(map => map.set(newContextId, newContext!));
        // Switch the current context ID
        currentContextId.set(newContextId);
        console.log(`Switched current context to ${newContextId}`);

        // TODO: Update viewport transform to center on the new focal node
        // TODO: Update currentFocalAbsoluteTransform store
    } else {
         console.error("Failed to load or create the new context object.");
    }
}


// --- Initialization ---

async function loadDataFromPersistence() {
    // Activate the default tool initially
    get(currentTool)?.activate();

    if (localAdapter) {
        try {
            // Load DataNodes
            const persistedNodes = await localAdapter.getNodes();
            const nodesMap = new Map<NodeId, DataNode>();
            if (persistedNodes && persistedNodes.length > 0) {
                persistedNodes.forEach((node: DataNode) => {
                    nodesMap.set(node.id, node);
                });
                console.log(`Loaded ${persistedNodes.length} nodes from persistence.`);
            } else {
                console.log("No nodes found in persistence.");
            }

            // Ensure Root Node exists
            if (!nodesMap.has(ROOT_NODE_ID)) {
                console.log("Root node not found, creating...");
                const now = Date.now();
                const rootNodeData: DataNode = {
                    id: ROOT_NODE_ID,
                    ntype: 'root', // Special type
                    createdAt: now,
                    modifiedAt: now,
                    path: '/', // Root path
                    attributes: { name: 'Root' },
                };
                nodesMap.set(ROOT_NODE_ID, rootNodeData);
                await localAdapter.saveNode(rootNodeData); // Save the new root node
            }
            nodes.set(nodesMap); // Set the potentially updated map

            // Load Edges
            const persistedEdges = await localAdapter.getEdges(); // Use getEdges
            if (persistedEdges && persistedEdges.length > 0) {
                const edgesMap = new Map<EdgeId, KartaEdge>();
                persistedEdges.forEach((edge: KartaEdge) => { // Use KartaEdge type
                    edgesMap.set(edge.id, edge);
                });
                edges.set(edgesMap); // Set the whole map
                console.log(`Loaded ${persistedEdges.length} edges from persistence.`);
            } else {
                console.log("No edges found in persistence.");
            }

            // Load Contexts from persistence
            // TODO: This needs a proper way to calculate focal transforms for all contexts, potentially recursively.
            // For initial load simplification, assume all contexts are relative to root (0,0,1,0).
            const dummyFocalTransforms = new Map<NodeId, AbsoluteTransform>();
            // We don't know all context IDs beforehand, but getContexts will default to ROOT_TRANSFORM if an ID is missing.
            // A better approach might be needed later.
            const persistedContexts = await localAdapter.getContexts(dummyFocalTransforms); // Pass dummy map

            const contextsMap = new Map<NodeId, Context>();
            if (persistedContexts && persistedContexts.length > 0) {
                 // Contexts returned by getContexts now have absolute coordinates (calculated using dummy ROOT_TRANSFORM)
                persistedContexts.forEach((context) => {
                    contextsMap.set(context.id, context);
                });
                console.log(`Loaded ${persistedContexts.length} contexts from persistence.`);
            } else {
                console.log("No contexts found in persistence.");
            }

            // Ensure the Root context exists
            const rootCtxId = ROOT_NODE_ID; // Use the constant
            if (!contextsMap.has(rootCtxId)) {
                 console.log(`Creating initial context: ${rootCtxId}`);
                 const newRootCtx: Context = { id: rootCtxId, viewNodes: new Map() };
                 contextsMap.set(rootCtxId, newRootCtx);
                 // Persist the newly created empty root context
                 // Use default ROOT_TRANSFORM as its own focal transform is (0,0)
                 await localAdapter.saveContext(newRootCtx);
            }

            // Populate Root context with ViewNodes for loaded DataNodes if they don't exist yet
            // This logic might need refinement depending on how nodes should appear initially
            const currentNodesMap = get(nodes); // Use the latest nodes map (includes root)
            const rootCtx = contextsMap.get(rootCtxId)!; // We know it exists now
            let rootContextWasModified = false; // Flag to track if we added defaults

            currentNodesMap.forEach((dataNode) => {
                // Special handling for the Root Node itself
                if (dataNode.id === rootCtxId) {
                    if (!rootCtx.viewNodes.has(dataNode.id)) {
                        console.log(`Adding default ViewNode for Root Node (${dataNode.id}) to context ${rootCtxId}`);
                        const rootViewNode: ViewNode = {
                            id: dataNode.id,
                            x: 0, // Root is always at origin
                            y: 0,
                            width: 150, // Make root slightly larger?
                            height: 100,
                            scale: 1,
                            rotation: 0,
                        };
                        rootCtx.viewNodes.set(dataNode.id, rootViewNode);
                        rootContextWasModified = true; // Mark as modified
                    }
                } else {
                    // Handle other nodes
                    if (!rootCtx.viewNodes.has(dataNode.id)) {
                        // Only add if it doesn't exist in the loaded root context data
                        console.log(`Adding default ViewNode for ${dataNode.id} to context ${rootCtxId}`);
                    const defaultViewNode: ViewNode = {
                        id: dataNode.id,
                        x: Math.random() * 500 - 250, // Random initial absolute position
                        y: Math.random() * 300 - 150, // Random position for non-root nodes
                        width: 100, // Default size
                        height: 100,
                        scale: 1,
                        rotation: 0,
                    };
                        rootCtx.viewNodes.set(dataNode.id, defaultViewNode);
                        rootContextWasModified = true; // Mark as modified
                    }
                }
            });

            // If we added default ViewNodes (including the root's), save the updated root context
            if (rootContextWasModified) {
                console.log(`Root context ${rootCtxId} was modified with defaults, saving...`);
                // Use default ROOT_TRANSFORM as its own focal transform is (0,0)
                await localAdapter.saveContext(rootCtx);
            }

            // Set the contexts store with loaded and potentially created/updated root context
            contexts.set(contextsMap);

            // Ensure currentContextId is set to Root Node ID after load
            currentContextId.set(rootCtxId);
        } catch (error) {
            console.error("Error loading data from persistence:", error);
            // console.log("Creating initial nodes and edges instead.");
            // createInitialNodes(); // Avoid creating initial nodes on error for now
        }
    } else {
        console.log("localAdapter not initialized, skipping load from persistence in SSR.");
        // createInitialNodes(); // Avoid creating initial nodes in SSR
         // Ensure default context exists even without persistence
         const rootCtxId = ROOT_NODE_ID;
         contexts.update(ctxMap => {
             if (!ctxMap.has(rootCtxId)) {
                  console.log(`Creating initial context (no persistence): ${rootCtxId}`);
                  ctxMap.set(rootCtxId, { id: rootCtxId, viewNodes: new Map() });
             }
             // Ensure ViewNodes exist for any already loaded DataNodes
             const rootCtx = ctxMap.get(rootCtxId)!;
             get(nodes).forEach((dataNode) => {
                 // Don't add root to its own context
                 if (dataNode.id === rootCtxId) return;
                 if (!rootCtx.viewNodes.has(dataNode.id)) {
                     const defaultViewNode: ViewNode = { id: dataNode.id, x: 0, y: 0, width: 100, height: 100, scale: 1, rotation: 0 };
                     rootCtx.viewNodes.set(dataNode.id, defaultViewNode);
                 }
             });
             return ctxMap;
         });
    }
}

// Removed createInitialNodes function

// Run initialization only in browser environment
if (typeof window !== 'undefined' ) {
    loadDataFromPersistence(); // Renamed function
}

// TODO: Implement saving/loading of Contexts (ViewNode layouts)
