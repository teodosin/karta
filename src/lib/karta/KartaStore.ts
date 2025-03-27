import { writable, get } from 'svelte/store';
import { tweened } from 'svelte/motion';
import { cubicOut } from 'svelte/easing';

// --- Types ---

export interface ViewNodeLayout {
	x: number;
	y: number;
	// scale, rotation etc. later
}

export interface DataNode {
	id: string; // Using simple string ID for now, will be UUID later
	label: string; // Example property
	// ntype, other attributes later
}

export interface Edge {
	id: string;
	source: string; // Source Node ID
	target: string; // Target Node ID
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
export const currentMode = writable<'move' | 'connect'>('move');
export const isConnecting = writable<boolean>(false);
export const connectionSourceNodeId = writable<NodeId | null>(null);
export const tempLineTargetPosition = writable<{ x: number; y: number } | null>(null);


// --- Store Actions ---

let nodeCounter = 0; // Simple ID generation for offline mode
let edgeCounter = 0; // Simple ID generation

export function createNodeAtPosition(canvasX: number, canvasY: number, labelPrefix = 'Node') {
	const newNodeId: NodeId = `node-${nodeCounter++}`;
	const newNode: DataNode = {
		id: newNodeId,
		label: `${labelPrefix} ${nodeCounter}`,
	};
	const newLayout: ViewNodeLayout = {
		x: canvasX,
		y: canvasY,
	};

	nodes.update(n => n.set(newNodeId, newNode));
	layout.update(l => l.set(newNodeId, newLayout));

	console.log(`Created node ${newNodeId} at (${canvasX}, ${canvasY})`);
    // TODO: Trigger persistence later
}

export function updateNodeLayout(nodeId: NodeId, newX: number, newY: number) {
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
    // TODO: Trigger batched persistence later
}


export function createEdge(sourceId: NodeId, targetId: NodeId) {
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
    // TODO: Trigger persistence later
}


export function startConnectionProcess(nodeId: NodeId) {
    if (get(currentMode) !== 'connect') return;
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
function createInitialNodes() {
	const gridSize = 3;
	const spacing = 150;
	const startX = 100;
	const startY = 100;

	for (let i = 0; i < gridSize; i++) {
		for (let j = 0; j < gridSize; j++) {
			const x = startX + j * spacing;
			const y = startY + i * spacing;
            // Use internal counter logic, rely on createNodeAtPosition
            createNodeAtPosition(x, y, `Node`);
		}
	}
     // Reset counter after initial setup
    nodeCounter = get(nodes).size;
}

// Run initialization only if nodes map is empty
if (typeof window !== 'undefined' && get(nodes).size === 0) { // Check for browser environment
    createInitialNodes();
}