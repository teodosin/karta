import { writable, get, derived } from 'svelte/store';
import { Tween } from 'svelte/motion';
import { cubicOut } from 'svelte/easing';
import type { ViewportSettings, AbsoluteTransform, ViewNode } from '../types/types';
import { currentContextId, currentViewNodes } from './ContextStore'; // Assuming ContextStore exports these

// Define default transform for root context or when focal node isn't visible
const DEFAULT_FOCAL_TRANSFORM: AbsoluteTransform = { x: 0, y: 0, scale: 1 }; // Represents target placement x, y, scale
const DEFAULT_VIEWPORT_SETTINGS: ViewportSettings = { scale: 1, posX: 0, posY: 0 }; // Default viewport state
const VIEWPORT_TWEEN_DURATION = 500;

// Store Definition
export const viewTransform = new Tween<ViewportSettings>( // Use ViewportSettings type
	{ ...DEFAULT_VIEWPORT_SETTINGS }, // Initialize with default
	{ duration: VIEWPORT_TWEEN_DURATION, easing: cubicOut } // Default tween settings
);

// Viewport Actions
/** Centers the viewport on a specific canvas coordinate, maintaining current scale. */
export function centerViewOnCanvasPoint(canvasX: number, canvasY: number) {
    const viewportEl = document.getElementById('viewport'); // Assuming viewport has this ID
    if (!viewportEl) {
    	console.error("[centerViewOnCanvasPoint] Viewport element not found.");
    	return;
    }
    const rect = viewportEl.getBoundingClientRect();
    const targetScale = 1; // Reset scale to 1 for centering actions
    const targetPosX = rect.width / 2 - canvasX * targetScale;
    const targetPosY = rect.height / 2 - canvasY * targetScale;

    // Set duration to 0 for immediate jump, stopping any current tween
    viewTransform.set({ scale: targetScale, posX: targetPosX, posY: targetPosY }, { duration: 0 });
    console.log(`[KartaStore] Centering view on canvas point (${canvasX}, ${canvasY}) at scale 1`);
}

/** Centers the viewport on the current focal node. */
export function centerOnFocalNode() {
    const focalNodeId = get(currentContextId);
    const focalViewNode = get(currentViewNodes).get(focalNodeId);

    if (focalViewNode) {
    	const nodeState = focalViewNode.state.current;
    	const centerX = nodeState.x + nodeState.width / 2;
    	const centerY = nodeState.y + nodeState.height / 2;
    	centerViewOnCanvasPoint(centerX, centerY);
    	console.log(`[KartaStore] Centering on focal node ${focalNodeId} at (${centerX}, ${centerY})`);
    } else {
    	console.warn(`[KartaStore] Cannot center on focal node: ViewNode ${focalNodeId} not found in current context.`);
    }
}

/** Calculates the bounding box of all nodes in the current context and adjusts the viewport to frame them. */
export function frameContext() {
    const viewportEl = document.getElementById('viewport');
    if (!viewportEl) {
    	console.error("[frameContext] Viewport element not found.");
    	return;
    }
    const rect = viewportEl.getBoundingClientRect();
    const nodesInContext = get(currentViewNodes);

    if (nodesInContext.size === 0) {
    	console.log("[frameContext] No nodes in context to frame.");
    	// Optionally reset to default view? For now, do nothing.
    	// viewTransform.set({ ...DEFAULT_VIEWPORT_SETTINGS }, { duration: 0 });
    	return;
    }

    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;

    nodesInContext.forEach(viewNode => {
    	const state = viewNode.state.current;
    	const nodeLeft = state.x - (state.width / 2) * state.scale;
    	const nodeRight = state.x + (state.width / 2) * state.scale;
    	const nodeTop = state.y - (state.height / 2) * state.scale;
    	const nodeBottom = state.y + (state.height / 2) * state.scale;

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
    	console.log("[frameContext] Bounding box has zero or negative dimensions, centering on first node.");
    	// Fallback: center on the first node found (or the focal node if it exists)
    	const firstNode = nodesInContext.values().next().value;
    	if (firstNode) {
    		const state = firstNode.state.current;
    		centerViewOnCanvasPoint(state.x + state.width / 2, state.y + state.height / 2);
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

    viewTransform.set({ scale: targetScale, posX: targetPosX, posY: targetPosY }, { duration: 0 });
    console.log(`[KartaStore] Framing context. Bounds: (${minX.toFixed(1)}, ${minY.toFixed(1)}) to (${maxX.toFixed(1)}, ${maxY.toFixed(1)}). New transform: scale=${targetScale.toFixed(2)}, pos=(${targetPosX.toFixed(1)}, ${targetPosY.toFixed(1)})`);

}

// Screen Coordinates Helper
export function screenToCanvasCoordinates(screenX: number, screenY: number, containerRect: DOMRect): { x: number; y: number } {
    const currentTransform = viewTransform.target;
	const canvasX = (screenX - containerRect.left - currentTransform.posX) / currentTransform.scale;
    const canvasY = (screenY - containerRect.top - currentTransform.posY) / currentTransform.scale;
	return { x: canvasX, y: canvasY };
}

// Export constants for use by other stores
export { DEFAULT_FOCAL_TRANSFORM, DEFAULT_VIEWPORT_SETTINGS, VIEWPORT_TWEEN_DURATION };