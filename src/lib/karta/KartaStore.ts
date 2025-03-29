import { writable, get } from 'svelte/store';
import { tweened } from 'svelte/motion';
import { cubicOut } from 'svelte/easing';
import { localAdapter } from '../util/LocalAdapter';
// Import the new types from types.ts
import type { DataNode, KartaEdge, ViewNode, Context, Tool, NodeId, EdgeId } from '../types/types';
import { MoveTool } from '../tools/MoveTool';
import { ConnectTool } from '../tools/ConnectTool';
import { ContextTool } from '../tools/ContextTool';
import { v4 as uuidv4 } from 'uuid'; // Import uuid for ID generation

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
export const currentContextId = writable<NodeId>('global_context'); // ID of the currently viewed context (focal node)

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


	console.log(`Created node ${newNodeId} (${newNodeData.attributes.name}) in context ${contextId}`);

    // Persist the DataNode
    if (localAdapter) {
        try {
            await localAdapter.saveNode(newNodeData);
        } catch (error) {
            console.error("Error saving node:", error);
        }
    } else {
        console.warn("LocalAdapter not initialized, persistence disabled in SSR.");
    }
    // TODO: Add persistence for Context/ViewNode updates later
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

                // Persist the updated context
                if (localAdapter) {
                    localAdapter.saveContext(currentCtx).catch(err => {
                        console.error(`Error saving context ${contextId}:`, err);
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


// --- Initialization ---

async function loadDataFromPersistence() {
    // Activate the default tool initially
    get(currentTool)?.activate();

    if (localAdapter) {
        try {
            // Load DataNodes
            const persistedNodes = await localAdapter.getNodes();
            if (persistedNodes && persistedNodes.length > 0) {
                const nodesMap = new Map<NodeId, DataNode>();
                persistedNodes.forEach((node: DataNode) => { // Use correct DataNode type
                    nodesMap.set(node.id, node);
                });
                nodes.set(nodesMap); // Set the whole map for reactivity
                console.log(`Loaded ${persistedNodes.length} nodes from persistence.`);
            } else {
                console.log("No nodes found in persistence.");
                // createInitialNodes(); // Don't create initial nodes if none found yet
            }

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
            const persistedContexts = await localAdapter.getContexts();
            const contextsMap = new Map<NodeId, Context>();
            if (persistedContexts && persistedContexts.length > 0) {
                persistedContexts.forEach((context) => {
                    contextsMap.set(context.id, context);
                });
                console.log(`Loaded ${persistedContexts.length} contexts from persistence.`);
            } else {
                console.log("No contexts found in persistence.");
            }

            // Ensure the default context exists
            const currentCtxId = get(currentContextId); // Usually 'global_context' initially
            if (!contextsMap.has(currentCtxId)) {
                 console.log(`Creating initial context: ${currentCtxId}`);
                 contextsMap.set(currentCtxId, { id: currentCtxId, viewNodes: new Map() });
            }

            // Populate default context with ViewNodes for loaded DataNodes, using loaded positions if available
            const loadedNodesMap = get(nodes); // Get the map of loaded DataNodes
            const defaultCtx = contextsMap.get(currentCtxId)!; // We know it exists now

            loadedNodesMap.forEach((dataNode) => {
                if (!defaultCtx.viewNodes.has(dataNode.id)) {
                    // Only add if it doesn't exist from loaded context data
                    console.log(`Adding default ViewNode for ${dataNode.id} to context ${currentCtxId}`);
                    const defaultViewNode: ViewNode = {
                        id: dataNode.id,
                        x: Math.random() * 500, // Random initial position for now
                        y: Math.random() * 300,
                        width: 100,
                        height: 100,
                        scale: 1,
                        rotation: 0,
                    };
                    defaultCtx.viewNodes.set(dataNode.id, defaultViewNode);
                }
            });

            // Set the contexts store with loaded and potentially updated default context
            contexts.set(contextsMap);


        } catch (error) {
            console.error("Error loading data from persistence:", error);
            // console.log("Creating initial nodes and edges instead.");
            // createInitialNodes(); // Avoid creating initial nodes on error for now
        }
    } else {
        console.log("localAdapter not initialized, skipping load from persistence in SSR.");
        // createInitialNodes(); // Avoid creating initial nodes in SSR
         // Ensure default context exists even without persistence
         const currentCtxId = get(currentContextId);
         contexts.update(ctxMap => {
             if (!ctxMap.has(currentCtxId)) {
                  console.log(`Creating initial context (no persistence): ${currentCtxId}`);
                  ctxMap.set(currentCtxId, { id: currentCtxId, viewNodes: new Map() });
             }
             // Ensure ViewNodes exist for any already loaded DataNodes (e.g., in SSR scenario before persistence check)
             const defaultCtx = ctxMap.get(currentCtxId)!;
             get(nodes).forEach((dataNode) => {
                 if (!defaultCtx.viewNodes.has(dataNode.id)) {
                     const defaultViewNode: ViewNode = { id: dataNode.id, x: 0, y: 0, width: 100, height: 100, scale: 1, rotation: 0 };
                     defaultCtx.viewNodes.set(dataNode.id, defaultViewNode);
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
