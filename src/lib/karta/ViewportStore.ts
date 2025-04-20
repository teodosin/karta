// ViewportStore: Manages viewport state (pan/zoom) and related actions.

import { Tween } from 'svelte/motion';
import { cubicOut } from 'svelte/easing';
import { get } from 'svelte/store';
import type { ViewportSettings } from '../types/types';
// Import necessary stores when they are fully defined
// import { currentContextId, currentViewNodes } from './ContextStore'; // Placeholder

// Define default transform and tween settings - might move to constants later
const DEFAULT_VIEWPORT_SETTINGS: ViewportSettings = { scale: 1, posX: 0, posY: 0 };
const VIEWPORT_TWEEN_DURATION = 500;

export const viewTransform = new Tween<ViewportSettings>(
	{ ...DEFAULT_VIEWPORT_SETTINGS },
	{ duration: VIEWPORT_TWEEN_DURATION, easing: cubicOut }
);

// --- Viewport Actions ---

/** Centers the viewport on a specific canvas coordinate, maintaining current scale. */
export function centerViewOnCanvasPoint(canvasX: number, canvasY: number) {
	const viewportEl = document.getElementById('viewport'); // Assuming viewport has this ID
	if (!viewportEl) {
		console.error("[ViewportStore - centerViewOnCanvasPoint] Viewport element not found.");
		return;
	}
	const rect = viewportEl.getBoundingClientRect();
	const targetScale = 1; // Reset scale to 1 for centering actions
	const targetPosX = rect.width / 2 - canvasX * targetScale;
	const targetPosY = rect.height / 2 - canvasY * targetScale;

	// Set duration to 0 for immediate jump, stopping any current tween
	viewTransform.set({ scale: targetScale, posX: targetPosX, posY: targetPosY }, { duration: 0 });
	console.log(`[ViewportStore] Centering view on canvas point (${canvasX}, ${canvasY}) at scale 1`);
}

/** Centers the viewport on the current focal node. */
export function centerOnFocalNode() {
	// TODO: Replace with import from ContextStore when available
	const focalNodeId_placeholder = 'placeholder-focal-id';
	const currentViewNodes_placeholder = new Map();
	// const focalNodeId = get(currentContextId); // From ContextStore
	// const focalViewNode = get(currentViewNodes).get(focalNodeId); // From ContextStore

	const focalViewNode = currentViewNodes_placeholder.get(focalNodeId_placeholder); // Placeholder access

	if (focalViewNode) {
		const nodeState = focalViewNode.state.current; // Assuming ViewNode structure
		const centerX = nodeState.x + nodeState.width / 2;
		const centerY = nodeState.y + nodeState.height / 2;
		centerViewOnCanvasPoint(centerX, centerY);
		console.log(`[ViewportStore] Centering on focal node ${focalNodeId_placeholder} at (${centerX}, ${centerY})`);
	} else {
		console.warn(`[ViewportStore] Cannot center on focal node: ViewNode ${focalNodeId_placeholder} not found in current context.`);
	}
}

/** Calculates the bounding box of all nodes in the current context and adjusts the viewport to frame them. */
export function frameContext() {
	const viewportEl = document.getElementById('viewport');
	if (!viewportEl) {
		console.error("[ViewportStore - frameContext] Viewport element not found.");
		return;
	}
	const rect = viewportEl.getBoundingClientRect();

	// TODO: Replace with import from ContextStore when available
	const currentViewNodes_placeholder = new Map();
	// const nodesInContext = get(currentViewNodes); // From ContextStore
	const nodesInContext = currentViewNodes_placeholder; // Placeholder access

	if (nodesInContext.size === 0) {
		console.log("[ViewportStore - frameContext] No nodes in context to frame.");
		// Optionally reset to default view?
		// viewTransform.set({ ...DEFAULT_VIEWPORT_SETTINGS }, { duration: 0 });
		return;
	}

	let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;

	nodesInContext.forEach((viewNode: any) => { // Use any for placeholder
		const state = viewNode.state.current; // Assuming ViewNode structure
		// Assuming node position is center, adjust for width/height/scale
		const halfWidth = (state.width / 2) * state.scale;
		const halfHeight = (state.height / 2) * state.scale;
		const nodeLeft = state.x - halfWidth;
		const nodeRight = state.x + halfWidth;
		const nodeTop = state.y - halfHeight;
		const nodeBottom = state.y + halfHeight;

		minX = Math.min(minX, nodeLeft);
		minY = Math.min(minY, nodeTop);
		maxX = Math.max(maxX, nodeRight);
		maxY = Math.max(maxY, nodeBottom);
	});

	const boundsWidth = maxX - minX;
	const boundsHeight = maxY - minY;
	const boundsCenterX = minX + boundsWidth / 2;
	const boundsCenterY = minY + boundsHeight / 2;

	if (boundsWidth <= 0 || boundsHeight <= 0) {
		console.log("[ViewportStore - frameContext] Bounding box has zero or negative dimensions, centering on first node.");
		// Fallback: center on the first node found
		const firstNode = nodesInContext.values().next().value;
		if (firstNode) {
			const state = firstNode.state.current;
			centerViewOnCanvasPoint(state.x, state.y); // Center on node center
		} else {
			viewTransform.set({ ...DEFAULT_VIEWPORT_SETTINGS }, { duration: 0 }); // Reset if truly empty
		}
		return;
	}

	const padding = 0.1; // 10% padding
	const scaleX = rect.width / (boundsWidth * (1 + padding));
	const scaleY = rect.height / (boundsHeight * (1 + padding));
	const targetScale = Math.min(scaleX, scaleY, 2); // Limit max zoom to 2x

	const targetPosX = rect.width / 2 - boundsCenterX * targetScale;
	const targetPosY = rect.height / 2 - boundsCenterY * targetScale;

	viewTransform.set({ scale: targetScale, posX: targetPosX, posY: targetPosY }, { duration: VIEWPORT_TWEEN_DURATION });
	console.log(`[ViewportStore] Framing context. Bounds: (${minX.toFixed(1)}, ${minY.toFixed(1)}) to (${maxX.toFixed(1)}, ${maxY.toFixed(1)}). New transform: scale=${targetScale.toFixed(2)}, pos=(${targetPosX.toFixed(1)}, ${targetPosY.toFixed(1)})`);
}


// Screen Coordinates Helper
export function screenToCanvasCoordinates(screenX: number, screenY: number, containerRect: DOMRect): { x: number; y: number } {
	// Access the current, non-tweened value using .current
	const currentTransform = viewTransform.current;
	const canvasX = (screenX - containerRect.left - currentTransform.posX) / currentTransform.scale;
	const canvasY = (screenY - containerRect.top - currentTransform.posY) / currentTransform.scale;
	return { x: canvasX, y: canvasY };
}