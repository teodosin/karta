import { writable, get } from 'svelte/store';
import { tweened } from 'svelte/motion';
import { cubicOut } from 'svelte/easing';
import { localAdapter } from '../util/LocalAdapter';
import type { KartaNode, KartaEdge, Tool } from '../types/types';
import { MoveTool } from '../tools/MoveTool';
import { ConnectTool } from '../tools/ConnectTool';
import { ContextTool } from '../tools/ContextTool';

// --- Types ---

export interface ViewNodeLayout {
	x: number;
	y: number;
	// scale, rotation etc. later
}

// Extend KartaNode from types.ts
export interface DataNode extends KartaNode {
	label: string; // Example property, remove later
}

export interface Edge extends KartaEdge {
	// no extra properties for now - it should extend KartaEdge, not implement it
}

// Use string for ContextID for simplicity initially
type ContextId = string;
export type NodeId = string; // Export for use in components
type EdgeId = string;

// --- Store Definition ---

// Viewport State
export const viewTransform = tweened(
	{ scale: 1, posX: 0, posY: 0 },
	{ duration: 200, easing: cubicOut }
);

// Data State
export const nodes = writable<Map<NodeId, DataNode>>(new Map());
export const edges = writable<Map<EdgeId, Edge>>(new Map());
export const layout = writable<Map<NodeId, ViewNodeLayout>>(new Map());
export const currentContextId = writable<ContextId>('global_context');

// Interaction State
export const currentTool = writable<Tool>(new MoveTool()); // Default to MoveTool
export const isConnecting = writable<boolean>(false);
export const connectionSourceNodeId = writable<NodeId | null>(null);
export const tempLineTargetPosition = writable<{ x: number; y: number } | null>(null);


// --- Store Actions ---

let nodeCounter = 0; // Simple ID generation for offline mode
let edgeCounter = 0; // Simple ID generation

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

export async function createNodeAtPosition(canvasX: number, canvasY: number, labelPrefix = 'Node') {
	const newNodeId: NodeId = `node-${nodeCounter++}`;
	const newNode: DataNode = {
		id: newNodeId,
		ntype: 'text', // Default node type for now
		label: `${labelPrefix} ${nodeCounter}`,
		x: canvasX,
		y: canvasY,
	};
	const newLayout: ViewNodeLayout = {
		x: canvasX,
		y: canvasY,
	};

	nodes.update(n => n.set(newNodeId, newNode));
	layout.update(l => l.set(newNodeId, newLayout));

	console.log(`Created node ${newNodeId} at (${canvasX}, ${canvasY})`);
    if (localAdapter) {
        await localAdapter?.saveNode(newNode);
    } else {
        console.warn("LocalAdapter not initialized, persistence disabled in SSR.");
    }
}

export async function updateNodeLayout(nodeId: NodeId, newX: number, newY: number) {
    layout.update(l => {
        const nodeLayout = l.get(nodeId);
        if (nodeLayout) {
            // Create a new object to ensure reactivity if map itself isn't reassigned
            l.set(nodeId, { ...nodeLayout, x: newX, y: newY });
        } else {
             l.set(nodeId, { x: newX, y: newY });
        }
        return new Map(l); // Return a new map to ensure reactivity
    });
    const node = get(nodes).get(nodeId);
    if (node && localAdapter) {
        await localAdapter.saveNode({...node, x: newX, y: newY}); // Update node position in persistence
    }
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

    const newEdgeId: EdgeId = `edge-${edgeCounter++}`;
	const newEdge: Edge = {
		id: newEdgeId,
		source: sourceId,
		target: targetId,
	};

	edges.update(e => e.set(newEdgeId, newEdge));
	console.log(`Created edge ${newEdgeId} between ${sourceId} and ${targetId}`);
    if (localAdapter && newEdge) { // Added check for newEdge
        await localAdapter.saveEdge(newEdge);
    } else {
        console.warn("LocalAdapter not initialized, persistence disabled in SSR.");
    }
}


export function startConnectionProcess(nodeId: NodeId) {
    if (!(get(currentTool) instanceof ConnectTool)) return;
    if (get(isConnecting)) return;
    isConnecting.set(true);
    connectionSourceNodeId.set(nodeId);
    const sourceLayout = get(layout).get(nodeId);
    if (sourceLayout) {
         // Center of a 100x100 node
        tempLineTargetPosition.set({x: sourceLayout.x + 50, y: sourceLayout.y + 50});
    }
    console.log("Starting connection from:", nodeId);
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

async function loadNodesFromPersistence() {
    // Activate the default tool initially
    get(currentTool)?.activate();

    if (localAdapter) {
        try {
            const persistedNodes = await localAdapter.getNodes();
            if (persistedNodes && persistedNodes.length > 0) {
                persistedNodes.forEach((node: KartaNode) => {
                    nodes.update(n => n.set(node.id, node as DataNode));
                    layout.update(l => l.set(node.id, { x: node.x, y: node.y }));
                });
                nodeCounter = persistedNodes.length;
                console.log(`Loaded ${persistedNodes.length} nodes from persistence.`);
            } else {
                console.log("No nodes found in persistence, creating initial nodes.");
                createInitialNodes();
            }

            const persistedEdges = await localAdapter.loadEdges();
            if (persistedEdges && persistedEdges.length > 0) {
                persistedEdges.forEach((edge: Edge) => {
                    edges.update(e => e.set(edge.id, edge));
                });
                edgeCounter = persistedEdges.length;
                console.log(`Loaded ${persistedEdges.length} edges from persistence.`);
            } else {
                console.log("No edges found in persistence.");
            }


        } catch (error) {
            console.error("Error loading data from persistence:", error);
            console.log("Creating initial nodes and edges instead.");
            createInitialNodes();
        }
    } else {
        console.log("localAdapter not initialized, skipping load from persistence in SSR.");
        createInitialNodes();
    }
}


function createInitialNodes() {
	// No initial nodes needed now, loading from persistence or creating on demand
}


// Run initialization only in browser environment
if (typeof window !== 'undefined' ) {
    loadNodesFromPersistence();
}


// --- Persistence Loading (Initial attempt, refine later) ---
// TODO: Load nodes, edges, layouts from LocalAdapter on store init
// For now, just creating initial nodes if store is empty on load.
