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
// Store more initial state per node
let initialNodesData: {
	id: NodeId;
	initialViewNode: ViewNode; // Keep the original ViewNode reference
	initialState: TweenableNodeState; // Snapshot of state at drag start
	initialAnchorCanvasPos: { x: number; y: number }; // Opposite corner pos
    initialCenterCanvasPos: { x: number; y: number }; // Center pos
}[] = [];

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
	if (!isResizing || !draggedHandle || !viewportContainerRect) return;

	event.preventDefault();
	event.stopPropagation();

	const currentCanvasPos = screenToCanvasCoordinates(event.clientX, event.clientY, viewportContainerRect);
	const maintainAspectRatio = event.shiftKey;
    const symmetricalResize = event.altKey; // Check Alt/Option key

	initialNodesData.forEach(({ id, initialViewNode, initialState, initialAnchorCanvasPos, initialCenterCanvasPos }) => {
		let finalWidth: number;
		let finalHeight: number;
		let finalCenterX: number;
		let finalCenterY: number;
        const initialAspectRatio = initialState.width > 0 && initialState.height > 0 ? initialState.width / initialState.height : null;

        if (symmetricalResize) {
            // --- Symmetrical Resize (Center Fixed) ---
            let halfWidth = Math.abs(currentCanvasPos.x - initialCenterCanvasPos.x);
            let halfHeight = Math.abs(currentCanvasPos.y - initialCenterCanvasPos.y);

            // Apply constraints symmetrically
            const constrained = applyConstraints(halfWidth * 2, halfHeight * 2, initialAspectRatio, maintainAspectRatio);
            finalWidth = constrained.width;
            finalHeight = constrained.height;

            // Center remains the same
            finalCenterX = initialCenterCanvasPos.x;
            finalCenterY = initialCenterCanvasPos.y;

        } else {
            // --- Standard Resize (Opposite Corner Fixed Visually) ---
            const rawWidth = Math.abs(currentCanvasPos.x - initialAnchorCanvasPos.x);
            const rawHeight = Math.abs(currentCanvasPos.y - initialAnchorCanvasPos.y);

            // Apply constraints
            const constrained = applyConstraints(rawWidth, rawHeight, initialAspectRatio, maintainAspectRatio);
            finalWidth = constrained.width;
            finalHeight = constrained.height;

            // Calculate the change in dimensions
            const deltaWidth = finalWidth - initialState.width;
            const deltaHeight = finalHeight - initialState.height;

            // Calculate the shift needed for the center based on the handle dragged
            let shiftX = 0;
            let shiftY = 0;

            if (draggedHandle === 'tl') { shiftX = -deltaWidth / 2; shiftY = -deltaHeight / 2; }
            else if (draggedHandle === 'tr') { shiftX = deltaWidth / 2; shiftY = -deltaHeight / 2; }
            else if (draggedHandle === 'bl') { shiftX = -deltaWidth / 2; shiftY = deltaHeight / 2; }
            else { /* br */ shiftX = deltaWidth / 2; shiftY = deltaHeight / 2; }

            // Calculate final center position by shifting the initial center
            finalCenterX = initialCenterCanvasPos.x + shiftX;
            finalCenterY = initialCenterCanvasPos.y + shiftY;
        }


		// --- Apply Update ---
		const finalTargetState: TweenableNodeState = {
			...initialState, // Preserve scale, rotation etc.
			x: finalCenterX,
			y: finalCenterY,
			width: finalWidth,
			height: finalHeight,
		};

		// Update the tween directly
		const contextId = get(currentContextId);
		const currentCtx = get(contexts).get(contextId);
		const viewNode = currentCtx?.viewNodes.get(id);
		if (viewNode) {
			viewNode.state.set(finalTargetState, { duration: 0 });
		} else {
			console.warn(`[ResizeLogic] ViewNode ${id} not found in context ${contextId} during resize.`);
		}
	});

	// Trigger reactivity for the contexts store after updating all nodes
	contexts.update(map => map);
}


function handlePointerUp(event: PointerEvent) {
	if (!isResizing) return;
	event.preventDefault();
	event.stopPropagation();
	window.removeEventListener('pointermove', handlePointerMove);
	window.removeEventListener('pointerup', handlePointerUp);
	isResizing = false;
	initialNodesData = [];
	draggedHandle = null;
	viewportContainerRect = null;
	console.log('[ResizeLogic] Resize ended');
}

// --- Public Function ---

export function startResize(
	event: PointerEvent,
	handlePosition: 'tl' | 'tr' | 'bl' | 'br',
	nodesToResize: { id: NodeId; initialViewNode: ViewNode }[]
) {
	if (isResizing) return;

	isResizing = true;
	startPointerX = event.clientX;
	startPointerY = event.clientY;
	draggedHandle = handlePosition;

    const containerEl = document.getElementById('viewport');
    if (!containerEl) {
        console.error("Viewport container not found for coordinate conversion.");
        isResizing = false;
        return;
    }
    viewportContainerRect = containerEl.getBoundingClientRect();

	// Calculate and store initial state for each node
	initialNodesData = nodesToResize.map(n => {
        const stateSnapshot = { ...n.initialViewNode.state.current }; // Take snapshot
        let anchorX = 0;
        let anchorY = 0;

        // Determine the fixed anchor corner's canvas position opposite the dragged handle
        // Note: Uses the state snapshot for calculations
        if (handlePosition === 'tl') { // Anchor is bottom-right
            anchorX = stateSnapshot.x + stateSnapshot.width / 2; // Use center + half-width/height for corners
            anchorY = stateSnapshot.y + stateSnapshot.height / 2;
        } else if (handlePosition === 'tr') { // Anchor is bottom-left
            anchorX = stateSnapshot.x - stateSnapshot.width / 2;
            anchorY = stateSnapshot.y + stateSnapshot.height / 2;
        } else if (handlePosition === 'bl') { // Anchor is top-right
            anchorX = stateSnapshot.x + stateSnapshot.width / 2;
            anchorY = stateSnapshot.y - stateSnapshot.height / 2;
        } else { // handlePosition === 'br', Anchor is top-left
            anchorX = stateSnapshot.x - stateSnapshot.width / 2;
            anchorY = stateSnapshot.y - stateSnapshot.height / 2;
        }

        return {
            id: n.id,
            initialViewNode: n.initialViewNode,
            initialState: stateSnapshot, // Store the snapshot
            initialAnchorCanvasPos: { x: anchorX, y: anchorY },
            initialCenterCanvasPos: { x: stateSnapshot.x, y: stateSnapshot.y } // Store initial center
        };
    });

	console.log(`[ResizeLogic] Resize started. Handle: ${handlePosition}, Nodes:`, initialNodesData.map(n => n.id));

	window.addEventListener('pointermove', handlePointerMove);
	window.addEventListener('pointerup', handlePointerUp);

	event.preventDefault();
    event.stopPropagation();
}