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

    // --- Name Incrementing Logic (Phase 5) ---
    let baseName = attributes.name || ntype; // Use provided name or default to ntype
    let finalName = baseName;
    let counter = 2;
    if (localAdapter) {
        // Check if the base name exists
        while (await localAdapter.checkNameExists(finalName)) {
            finalName = `${baseName}${counter}`;
            counter++;
        }
    } else {
        // Fallback if adapter isn't ready (e.g., SSR) - might lead to duplicates later
        console.warn("LocalAdapter not ready, cannot check for duplicate names.");
    }
    console.log(`[KartaStore] Determined final node name: ${finalName}`);
    // --- End Name Incrementing Logic ---

	const newNodeData: DataNode = {
		id: newNodeId,
		ntype: ntype,
        createdAt: now,
        modifiedAt: now,
        path: `/node_${newNodeId}`, // Added: Simple default path
		attributes: {
            ...attributes, // Merge provided attributes first
            name: finalName, // Set the potentially incremented name
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

// Define default transform for root context or when focal node isn't visible
const DEFAULT_FOCAL_TRANSFORM: AbsoluteTransform = { x: 0, y: 0, scale: 1, rotation: 0 };

export async function switchContext(newContextId: NodeId) {
    console.log(`[switchContext] Attempting to switch to context: ${newContextId}`);
    const oldContextId = get(currentContextId);
    if (newContextId === oldContextId) {
        console.log(`[switchContext] Already in context ${newContextId}. Aborting.`);
        return; // Already in the target context
    }

    if (!localAdapter) {
        console.error("[switchContext] LocalAdapter not available. Cannot switch context.");
        return;
    }

    // --- Step 1: Save Old Context (Asynchronous save will be Phase 4) ---
    const oldContext = get(contexts).get(oldContextId);
    if (oldContext) {
        console.log(`[switchContext] Saving old context: ${oldContextId} (async)`);
        // Make save asynchronous (Phase 4) - don't await
        localAdapter.saveContext(oldContext).then(() => {
            console.log(`[switchContext] Old context ${oldContextId} saved successfully (async).`);
        }).catch(error => {
            console.error(`[switchContext] Error saving old context ${oldContextId} (async):`, error);
            // Continue the switch even if saving fails for now
        }); // <-- Added missing closing parenthesis and semicolon
    } else {
        console.warn(`[switchContext] Old context ${oldContextId} not found in memory store.`);
    }

    // --- Step 2: Determine New Focal Transform ---
    let newFocalAbsTransform: AbsoluteTransform;
    const currentViewNodes = get(contexts).get(oldContextId)?.viewNodes; // Check the *old* context's viewNodes
    const targetFocalViewNode = currentViewNodes?.get(newContextId);

    if (targetFocalViewNode) {
        // Target focal node is visible in the current context
        newFocalAbsTransform = {
            x: targetFocalViewNode.x,
            y: targetFocalViewNode.y,
            scale: targetFocalViewNode.scale,
            rotation: targetFocalViewNode.rotation
        };
        console.log(`[switchContext] Target focal node ${newContextId} found in current view. Transform:`, newFocalAbsTransform);
    } else {
        // Target focal node is NOT visible (non-visible jump or initial load)
        // Use a default transform (e.g., origin or center of screen)
        // For simplicity, using origin for now. Could use old focal position later.
        newFocalAbsTransform = { ...DEFAULT_FOCAL_TRANSFORM };
        console.log(`[switchContext] Target focal node ${newContextId} not visible. Using default transform:`, newFocalAbsTransform);
    }

    // --- Step 3: Load New Context Data ---
    let loadedContext: Context | undefined;
    let loadedDataNodes: Map<NodeId, DataNode> = new Map();
    let loadedEdges: Map<EdgeId, KartaEdge> = new Map();
    let contextNeedsCreation = false;

    try {
        console.log(`[switchContext] Loading context ${newContextId} with focal transform:`, newFocalAbsTransform);
        loadedContext = await localAdapter.getContext(newContextId, newFocalAbsTransform);

        if (loadedContext) {
            console.log(`[switchContext] Context ${newContextId} loaded from DB.`);
            const viewNodeIds = Array.from(loadedContext.viewNodes.keys());
            if (viewNodeIds.length > 0) {
                console.log(`[switchContext] Loading ${viewNodeIds.length} DataNodes for context ${newContextId}...`);
                loadedDataNodes = await localAdapter.getDataNodesByIds(viewNodeIds);
                console.log(`[switchContext] Loaded ${loadedDataNodes.size} DataNodes.`);

                console.log(`[switchContext] Loading Edges for context ${newContextId}...`);
                loadedEdges = await localAdapter.getEdgesByNodeIds(viewNodeIds);
                console.log(`[switchContext] Loaded ${loadedEdges.size} Edges.`);
            } else {
                 console.log(`[switchContext] Context ${newContextId} has no view nodes.`);
            }
        } else {
            console.log(`[switchContext] Context ${newContextId} not found in DB, creating new.`);
            contextNeedsCreation = true;
            // Ensure the focal DataNode exists if we're creating its context
            const focalDataNode = await localAdapter.getNode(newContextId);
            if (!focalDataNode) {
                 throw new Error(`Cannot create context for non-existent node: ${newContextId}`);
            }
            loadedDataNodes.set(newContextId, focalDataNode); // Add the focal node data

            // Create a basic context containing only the focal ViewNode at the calculated absolute position
            const focalViewNode: ViewNode = {
                id: newContextId,
                x: newFocalAbsTransform.x,
                y: newFocalAbsTransform.y,
                width: 100, // Default size
                height: 100,
                scale: newFocalAbsTransform.scale,
                rotation: newFocalAbsTransform.rotation,
            };
            loadedContext = { id: newContextId, viewNodes: new Map([[newContextId, focalViewNode]]) };

            // TODO: Implement default population logic (e.g., add connected nodes)? Deferred for now.

            // Save the newly created context immediately
            console.log(`[switchContext] Saving newly created context ${newContextId}...`);
            await localAdapter.saveContext(loadedContext);
        }
    } catch (error) {
        console.error(`[switchContext] Error loading or creating context ${newContextId}:`, error);
        return; // Abort switch if loading/creation fails
    }

    // --- Step 4: Update Stores (Mark-and-Sweep Logic for Phase 3) ---
    if (loadedContext) {
        // Get current state
        const currentNodes = get(nodes);
        const currentEdges = get(edges);

        // Mark all current nodes and edges for potential removal
        const nodesToRemove = new Set(currentNodes.keys());
        const edgesToRemove = new Set(currentEdges.keys());

        // Prepare maps for the next state
        const newNodes = new Map<NodeId, DataNode>();
        const newEdges = new Map<EdgeId, KartaEdge>();

        // Process loaded nodes: add to new map and unmark for removal
        for (const [nodeId, dataNode] of loadedDataNodes.entries()) {
            newNodes.set(nodeId, dataNode);
            nodesToRemove.delete(nodeId); // Keep this node
        }

        // Process loaded edges: add to new map and unmark for removal
        for (const [edgeId, edge] of loadedEdges.entries()) {
            // Ensure both source and target nodes are present in the new context's nodes
            if (newNodes.has(edge.source) && newNodes.has(edge.target)) {
                newEdges.set(edgeId, edge);
                edgesToRemove.delete(edgeId); // Keep this edge
            } else {
                 console.warn(`[switchContext] Edge ${edgeId} skipped: Source or target node not in new context.`);
            }
        }

        // Add back any nodes from the previous state that weren't marked for removal
        // (This handles nodes potentially shared across contexts but not directly part of the loaded context's viewNodes,
        // although currently getDataNodesByIds only fetches nodes in viewNodes. This might be redundant for now,
        // but could be useful if data loading logic changes).
        // Let's simplify: Mark-and-sweep only applies to what's *currently* displayed vs what *will be* displayed.
        // The loadedDataNodes and loadedEdges represent the complete state for the new context.

        console.log(`[switchContext] Mark-and-Sweep: Nodes to remove: ${nodesToRemove.size}, Edges to remove: ${edgesToRemove.size}`);

        // Set the new state for nodes and edges
        nodes.set(newNodes);
        edges.set(newEdges);

        // Update the contexts map (add/replace the loaded context's view data)
        // The 'contexts' store holds the ViewNode layout for each loaded context.
        contexts.update(map => map.set(newContextId, loadedContext!));

        // Switch the current context ID
        currentContextId.set(newContextId);
        console.log(`[switchContext] Switched current context ID to ${newContextId}`);
        console.log(`[switchContext] Stores updated. Nodes: ${get(nodes).size}, Edges: ${get(edges).size}, Contexts: ${get(contexts).size}`);

        // TODO: Update viewport transform to center on the new focal node (Phase 6 refinement)
        // TODO: Update currentFocalAbsoluteTransform store (if needed)
    } else {
         console.error("[switchContext] Failed to load or create the new context object. Stores not updated.");
    }
}


// --- Initialization ---

async function initializeStores() {
    // Activate the default tool initially
    get(currentTool)?.activate();

    if (localAdapter) {
        try {
            // 1. Ensure Root DataNode exists in DB (critical for first load)
            const rootNodeExists = await localAdapter.getNode(ROOT_NODE_ID);
            if (!rootNodeExists) {
                console.log("Root node not found in DB, creating...");
                const now = Date.now();
                const rootNodeData: DataNode = {
                    id: ROOT_NODE_ID,
                    ntype: 'root', // Special type
                    createdAt: now,
                    modifiedAt: now,
                    path: '/', // Root path
                    attributes: { name: 'Root' },
                };
                await localAdapter.saveNode(rootNodeData);
            } else {
                console.log("Root node found in DB.");
            }

            // 2. Load Root Context directly for initialization
            console.log("Loading initial Root context data...");
            const rootContextId = ROOT_NODE_ID;
            let rootContext = await localAdapter.getContext(rootContextId, DEFAULT_FOCAL_TRANSFORM);
            let rootDataNodes = new Map<NodeId, DataNode>();
            let rootEdges = new Map<EdgeId, KartaEdge>();

            if (!rootContext) {
                console.log("Root context not found in DB, creating...");
                // Ensure Root DataNode is loaded/available
                const rootDataNode = await localAdapter.getNode(rootContextId);
                if (!rootDataNode) throw new Error("Root DataNode missing during Root Context creation.");
                rootDataNodes.set(rootContextId, rootDataNode);

                // Create Root ViewNode
                const rootViewNode: ViewNode = {
                    id: rootContextId,
                    x: DEFAULT_FOCAL_TRANSFORM.x,
                    y: DEFAULT_FOCAL_TRANSFORM.y,
                    width: 150, height: 100, // Default size
                    scale: DEFAULT_FOCAL_TRANSFORM.scale,
                    rotation: DEFAULT_FOCAL_TRANSFORM.rotation,
                };
                rootContext = { id: rootContextId, viewNodes: new Map([[rootContextId, rootViewNode]]) };
                await localAdapter.saveContext(rootContext); // Save the new root context
            } else {
                 console.log("Root context loaded from DB.");
                 // Load associated DataNodes and Edges
                 const viewNodeIds = Array.from(rootContext.viewNodes.keys());
                 if (viewNodeIds.length > 0) {
                     rootDataNodes = await localAdapter.getDataNodesByIds(viewNodeIds);
                     rootEdges = await localAdapter.getEdgesByNodeIds(viewNodeIds);
                 }
            }

            // 3. Set initial store state
            nodes.set(rootDataNodes);
            edges.set(rootEdges);
            contexts.set(new Map([[rootContextId, rootContext]])); // Initialize contexts map with Root
            currentContextId.set(rootContextId); // Set current context

            console.log(`Initialization complete. Context: ${rootContextId}, Nodes: ${rootDataNodes.size}, Edges: ${rootEdges.size}`);

        } catch (error) {
            console.error("Error during store initialization:", error);
            // Handle initialization error appropriately, maybe set default empty state?
            nodes.set(new Map());
            edges.set(new Map());
            contexts.set(new Map());
            currentContextId.set(ROOT_NODE_ID); // Default to root even on error?
        }
    } else {
        console.warn("LocalAdapter not initialized, skipping initialization in SSR.");
        // Set default empty state for SSR or environments without persistence
        nodes.set(new Map());
        edges.set(new Map());
        contexts.set(new Map());
        currentContextId.set(ROOT_NODE_ID);
    }
}

// Run initialization only in browser environment
if (typeof window !== 'undefined' ) {
    initializeStores(); // Renamed function
}
