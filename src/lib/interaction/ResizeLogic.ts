import type { ViewNode, NodeId, TweenableNodeState } from '$lib/types/types';
import { contexts, currentContextId, viewTransform, screenToCanvasCoordinates } from '$lib/karta/KartaStore';
import { get } from 'svelte/store';

const MIN_NODE_DIMENSION = 20;

// State variables for the drag operation
let isResizing = false;
let startPointerX = 0; // Screen coordinates at start (Declaration added back)
let startPointerY = 0; // Screen coordinates at start (Declaration added back)
let draggedHandle: 'tl' | 'tr' | 'bl' | 'br' | null = null;
let viewportContainerRect: DOMRect | null = null;
let primaryNodeId: NodeId | null = null; // ID of the node whose handle was clicked (might not be needed with bounds approach)
// Store initial state per node
let initialNodesData: Map<NodeId, {
	initialViewNode: ViewNode;
	initialState: TweenableNodeState; // Snapshot of state at drag start
}> = new Map();
// Store initial bounding box info
let initialBounds: { minX: number; minY: number; maxX: number; maxY: number; width: number; height: number; } | null = null;
let initialBoundsAnchorPos: { x: number; y: number } | null = null; // Anchor point of the bounding box

// --- Helper Functions ---

function applyConstraints(
    rawWidth: number,
    rawHeight: number,
    initialAspectRatio: number | null, // Pass null if aspect ratio shouldn't be maintained
    maintainAspectRatio: boolean
): { width: number; height: number } {
    // 1. Minimum Dimensions
    let finalWidth = Math.max(MIN_NODE_DIMENSION, rawWidth);
    let finalHeight = Math.max(MIN_NODE_DIMENSION, rawHeight);

    // 2. Aspect Ratio Lock (applied AFTER minimum dimensions)
    if (maintainAspectRatio && initialAspectRatio !== null && initialAspectRatio > 0) {
        const currentRawAspectRatio = rawHeight === 0 ? initialAspectRatio : rawWidth / rawHeight;

        if (currentRawAspectRatio > initialAspectRatio) {
            // Width is the dominant change relative to aspect ratio; adjust height based on finalWidth
            finalHeight = finalWidth / initialAspectRatio;
        } else {
            // Height is the dominant change relative to aspect ratio; adjust width based on finalHeight
            finalWidth = finalHeight * initialAspectRatio;
        }

        // Re-check minimum dimensions after aspect ratio adjustment
        if (finalWidth < MIN_NODE_DIMENSION) {
            finalWidth = MIN_NODE_DIMENSION;
            finalHeight = finalWidth / initialAspectRatio;
        }
        if (finalHeight < MIN_NODE_DIMENSION) {
            finalHeight = MIN_NODE_DIMENSION;
            finalWidth = finalHeight * initialAspectRatio;
        }
    }
    return { width: finalWidth, height: finalHeight };
}


// --- Event Handlers ---

function handlePointerMove(event: PointerEvent) {
	// Use initialBounds and initialBoundsAnchorPos now
	if (!isResizing || !draggedHandle || !viewportContainerRect || !initialBounds || !initialBoundsAnchorPos) return;

	event.preventDefault();
	event.stopPropagation();

	const currentCanvasPos = screenToCanvasCoordinates(event.clientX, event.clientY, viewportContainerRect);
	const maintainAspectRatio = event.shiftKey;
	// const symmetricalResize = event.altKey; // Symmetrical resize based on bounds center is complex, defer for now.

	// --- Calculate Transformation based on Bounding Box Anchor ---
	const initialBoundsAspectRatio = initialBounds.width > 0 && initialBounds.height > 0 ? initialBounds.width / initialBounds.height : null;

	// 1. Calculate target dimensions of the bounding box based on cursor distance from the fixed anchor
	const rawTargetBoundsWidth = Math.abs(currentCanvasPos.x - initialBoundsAnchorPos.x);
	const rawTargetBoundsHeight = Math.abs(currentCanvasPos.y - initialBoundsAnchorPos.y);

	// 2. Apply constraints (min dimensions, aspect ratio) to the bounding box size
	const constrainedBounds = applyConstraints(rawTargetBoundsWidth, rawTargetBoundsHeight, initialBoundsAspectRatio, maintainAspectRatio);
	const finalBoundsWidth = constrainedBounds.width;
	const finalBoundsHeight = constrainedBounds.height;

	// 3. Calculate scale factors based on the change in bounding box size
	const scaleX = initialBounds.width === 0 ? 1 : finalBoundsWidth / initialBounds.width;
	const scaleY = initialBounds.height === 0 ? 1 : finalBoundsHeight / initialBounds.height;

	// 4. Determine the new position of the bounding box anchor point.
	//    For standard resize, the anchor point itself doesn't move relative to the canvas.
	//    The *opposite* corner moves based on the calculated final dimensions.
	const newBoundsAnchorX = initialBoundsAnchorPos.x;
	const newBoundsAnchorY = initialBoundsAnchorPos.y;

	// --- Apply Transformation to ALL Selected Nodes ---
	const contextId = get(currentContextId);
	const currentCtx = get(contexts).get(contextId);

	if (!currentCtx) {
		console.error(`[ResizeLogic] Current context ${contextId} not found.`);
		handlePointerUp(event);
		return;
	}

	// Assign to a new const *after* the null check at the function start to satisfy TypeScript
	const checkedBounds = initialBounds; // checkedBounds is now guaranteed non-null
	const checkedBoundsAnchor = initialBoundsAnchorPos; // guaranteed non-null

	initialNodesData.forEach(({ initialViewNode, initialState }, id) => {
		const viewNode = currentCtx.viewNodes.get(id);
		if (!viewNode) {
			console.warn(`[ResizeLogic] ViewNode ${id} not found in context ${contextId} during resize update.`);
			return; // Skip this node
		}

		// 5a. Calculate node's initial position relative to the GROUP's anchor point
		const initialRelX = initialState.x - checkedBoundsAnchor.x;
		const initialRelY = initialState.y - checkedBoundsAnchor.y;

		// 5b. Calculate the final absolute center position by scaling the relative position
		//     and adding it to the NEW (which is the same as initial) anchor position.
		const finalCenterX = newBoundsAnchorX + initialRelX * scaleX;
		const finalCenterY = newBoundsAnchorY + initialRelY * scaleY;

		// 5c. Calculate final dimensions by scaling initial dimensions
		const finalWidth = initialState.width * scaleX;
		const finalHeight = initialState.height * scaleY;

		const finalTargetState: TweenableNodeState = {
			...initialState, // Preserve original rotation etc.
			x: finalCenterX,
			y: finalCenterY,
			width: finalWidth,
			height: finalHeight,
			scale: initialState.scale // Keep individual node scale unchanged
		};

		// 6. Update the tween directly
		viewNode.state.set(finalTargetState, { duration: 0 });
	});

	// Trigger reactivity for the contexts store once after updating all nodes
	contexts.update(map => map);
}


function handlePointerUp(event: PointerEvent) {
	if (!isResizing) return;
	event.preventDefault();
	event.stopPropagation();
	window.removeEventListener('pointermove', handlePointerMove);
	window.removeEventListener('pointerup', handlePointerUp);
	isResizing = false;
	initialNodesData.clear(); // Use clear() for Map
	draggedHandle = null;
	viewportContainerRect = null;
	console.log('[ResizeLogic] Resize ended');
}

// --- Public Function ---

export function startResize(
	event: PointerEvent,
	handlePosition: 'tl' | 'tr' | 'bl' | 'br',
	clickedNodeId: NodeId, // Add the ID of the node whose handle was clicked
	nodesToResize: { id: NodeId; initialViewNode: ViewNode }[]
) {
	if (isResizing) return;

	isResizing = true;
	startPointerX = event.clientX;
	startPointerY = event.clientY;
	draggedHandle = handlePosition;
	primaryNodeId = clickedNodeId; // Store the primary node ID

    const containerEl = document.getElementById('viewport');
    if (!containerEl) {
        console.error("Viewport container not found for coordinate conversion.");
        isResizing = false;
        return;
    }
    viewportContainerRect = containerEl.getBoundingClientRect();

	// --- Calculate and store initial state for each node AND the bounding box ---
	initialNodesData.clear();
	initialBounds = null;
	initialBoundsAnchorPos = null;

	let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;

	nodesToResize.forEach(n => {
		const stateSnapshot = { ...n.initialViewNode.state.current }; // Take snapshot
		initialNodesData.set(n.id, {
			initialViewNode: n.initialViewNode,
			initialState: stateSnapshot,
		});

		// Update bounds based on node's corners
		const halfWidth = stateSnapshot.width / 2;
		const halfHeight = stateSnapshot.height / 2;
		minX = Math.min(minX, stateSnapshot.x - halfWidth);
		minY = Math.min(minY, stateSnapshot.y - halfHeight);
		maxX = Math.max(maxX, stateSnapshot.x + halfWidth);
		maxY = Math.max(maxY, stateSnapshot.y + halfHeight);
	});

	if (nodesToResize.length > 0) {
		initialBounds = { minX, minY, maxX, maxY, width: maxX - minX, height: maxY - minY };

		// Determine the fixed anchor corner of the BOUNDING BOX opposite the dragged handle
		if (handlePosition === 'tl') { initialBoundsAnchorPos = { x: initialBounds.maxX, y: initialBounds.maxY }; } // Anchor is bottom-right
		else if (handlePosition === 'tr') { initialBoundsAnchorPos = { x: initialBounds.minX, y: initialBounds.maxY }; } // Anchor is bottom-left
		else if (handlePosition === 'bl') { initialBoundsAnchorPos = { x: initialBounds.maxX, y: initialBounds.minY }; } // Anchor is top-right
		else { /* br */ initialBoundsAnchorPos = { x: initialBounds.minX, y: initialBounds.minY }; } // Anchor is top-left
	}
	// --- End initial state calculation ---

	console.log(`[ResizeLogic] Resize started. Handle: ${handlePosition}, Nodes:`, Array.from(initialNodesData.keys()), "Bounds:", initialBounds);

	// Add listeners AFTER storing initial data
	window.addEventListener('pointermove', handlePointerMove);
	window.addEventListener('pointerup', handlePointerUp);

	event.preventDefault();
    event.stopPropagation();
}