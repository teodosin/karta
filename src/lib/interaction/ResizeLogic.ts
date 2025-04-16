import type { ViewNode, NodeId, TweenableNodeState } from '$lib/types/types';
import { contexts, currentContextId, viewTransform, screenToCanvasCoordinates } from '$lib/karta/KartaStore'; // Import viewTransform and screenToCanvasCoordinates
import { get } from 'svelte/store';

const MIN_NODE_DIMENSION = 20; // Minimum width/height for a node

// State variables for the drag operation
let isResizing = false;
let startPointerX = 0; // Screen coordinates at start
let startPointerY = 0; // Screen coordinates at start
let initialNodesState: { id: NodeId; initialViewNode: ViewNode; initialAnchorCanvasPos: { x: number; y: number } }[] = []; // Store initial anchor canvas pos per node
let draggedHandle: 'tl' | 'tr' | 'bl' | 'br' | null = null;
let viewportContainerRect: DOMRect | null = null; // Store viewport rect for coordinate conversion

// --- Event Handlers ---

function handlePointerMove(event: PointerEvent) {
	if (!isResizing || !draggedHandle || !viewportContainerRect) return;

	event.preventDefault();
	event.stopPropagation();

	// Get current cursor position in canvas coordinates
	const currentCanvasPos = screenToCanvasCoordinates(event.clientX, event.clientY, viewportContainerRect);
	const maintainAspectRatio = event.shiftKey;

	initialNodesState.forEach(({ id, initialViewNode, initialAnchorCanvasPos }) => {
		const initialState = initialViewNode.state.current; // Original state at drag start
		let rawWidth: number;
		let rawHeight: number;
		let finalWidth: number;
		let finalHeight: number;
		let finalX: number;
		let finalY: number;

        // --- Calculate Raw Dimensions from Anchor and Cursor ---
        // This determines the size based purely on the vector from anchor to cursor
        rawWidth = Math.abs(currentCanvasPos.x - initialAnchorCanvasPos.x);
        rawHeight = Math.abs(currentCanvasPos.y - initialAnchorCanvasPos.y);

        // --- Apply Constraints ---

        // 1. Minimum Dimensions
        finalWidth = Math.max(MIN_NODE_DIMENSION, rawWidth);
        finalHeight = Math.max(MIN_NODE_DIMENSION, rawHeight);

        // 2. Aspect Ratio Lock (applied AFTER minimum dimensions)
		if (maintainAspectRatio && initialState.width > 0 && initialState.height > 0) {
			const initialAspectRatio = initialState.width / initialState.height;

            // Determine dominant change axis based on raw dimensions vs initial aspect ratio
            // Use the raw dimensions for this comparison to see which axis the user is 'pulling' more
            // Avoid division by zero if rawHeight is 0
            const currentRawAspectRatio = rawHeight === 0 ? initialAspectRatio : rawWidth / rawHeight;

            if (currentRawAspectRatio > initialAspectRatio) {
                // Width is the dominant change relative to aspect ratio; adjust height based on finalWidth
                finalHeight = finalWidth / initialAspectRatio;
            } else {
                // Height is the dominant change relative to aspect ratio; adjust width based on finalHeight
                finalWidth = finalHeight * initialAspectRatio;
            }

            // Re-check minimum dimensions after aspect ratio adjustment
            // If aspect ratio forces one dimension below minimum, recalculate based on the other axis hitting minimum
            if (finalWidth < MIN_NODE_DIMENSION) {
                finalWidth = MIN_NODE_DIMENSION;
                finalHeight = finalWidth / initialAspectRatio;
            }
            if (finalHeight < MIN_NODE_DIMENSION) {
                finalHeight = MIN_NODE_DIMENSION;
                finalWidth = finalHeight * initialAspectRatio;
            }
		}

        // --- Calculate Final Position (Top-Left Corner) based on Anchor and Final Dimensions ---
        // This ensures the anchor point remains stationary
        if (draggedHandle === 'br') { // Anchor: top-left
            finalX = initialAnchorCanvasPos.x;
            finalY = initialAnchorCanvasPos.y;
        } else if (draggedHandle === 'bl') { // Anchor: top-right
            finalX = initialAnchorCanvasPos.x - finalWidth;
            finalY = initialAnchorCanvasPos.y;
        } else if (draggedHandle === 'tr') { // Anchor: bottom-left
            finalX = initialAnchorCanvasPos.x;
            finalY = initialAnchorCanvasPos.y - finalHeight;
        } else { // draggedHandle === 'tl', Anchor: bottom-right
            finalX = initialAnchorCanvasPos.x - finalWidth;
            finalY = initialAnchorCanvasPos.y - finalHeight;
        }

		// --- Apply Update ---
		const finalTargetState: TweenableNodeState = {
			...initialState, // Preserve scale, rotation etc. from original state
			x: finalX,
			y: finalY,
			width: finalWidth,
			height: finalHeight,
            scale: initialState.scale, // Keep original scale
            rotation: initialState.rotation // Keep original rotation
		};

		// Update the tween directly and trigger reactivity
		const contextId = get(currentContextId);
		const currentCtx = get(contexts).get(contextId);
		const viewNode = currentCtx?.viewNodes.get(id);
		if (viewNode) {
			viewNode.state.set(finalTargetState, { duration: 0 }); // Update tween instantly
		} else {
			console.warn(`[ResizeLogic] ViewNode ${id} not found in context ${contextId} during resize.`);
		}
	});
	// Trigger reactivity for the contexts store after updating all nodes in the batch
	contexts.update(map => map);
}


function handlePointerUp(event: PointerEvent) {
	if (!isResizing) return;

	event.preventDefault();
	event.stopPropagation();

	// Clean up
	window.removeEventListener('pointermove', handlePointerMove);
	window.removeEventListener('pointerup', handlePointerUp);
	isResizing = false;
	initialNodesState = [];
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
	if (isResizing) return; // Prevent starting multiple resizes

	isResizing = true;
	startPointerX = event.clientX; // Still need screen coords for start point
	startPointerY = event.clientY;
	draggedHandle = handlePosition;

    // Get viewport container - assuming fixed ID for now
    const containerEl = document.getElementById('viewport'); // TODO: Make this more robust? Pass from component?
    if (!containerEl) {
        console.error("Viewport container not found for coordinate conversion.");
        isResizing = false;
        return;
    }
    viewportContainerRect = containerEl.getBoundingClientRect();

	// Calculate and store initial state including the fixed anchor's CANVAS position for each node
	initialNodesState = nodesToResize.map(n => {
        const initialState = n.initialViewNode.state.current;
        let anchorX = 0;
        let anchorY = 0;

        // Determine the fixed anchor corner's canvas position opposite the dragged handle
        if (handlePosition === 'tl') { // Anchor is bottom-right
            anchorX = initialState.x + initialState.width;
            anchorY = initialState.y + initialState.height;
        } else if (handlePosition === 'tr') { // Anchor is bottom-left
            anchorX = initialState.x;
            anchorY = initialState.y + initialState.height;
        } else if (handlePosition === 'bl') { // Anchor is top-right
            anchorX = initialState.x + initialState.width;
            anchorY = initialState.y;
        } else { // handlePosition === 'br', Anchor is top-left
            anchorX = initialState.x;
            anchorY = initialState.y;
        }

        return {
            id: n.id,
            initialViewNode: n.initialViewNode, // Keep original reference
            initialAnchorCanvasPos: { x: anchorX, y: anchorY }
        };
    });


	console.log(`[ResizeLogic] Resize started. Handle: ${handlePosition}, Nodes:`, initialNodesState.map(n => n.id));


	// Add global listeners
	window.addEventListener('pointermove', handlePointerMove);
	window.addEventListener('pointerup', handlePointerUp);

	// Prevent default browser drag behavior if needed
	event.preventDefault();
    event.stopPropagation();
}