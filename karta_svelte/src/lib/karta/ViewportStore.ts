import { writable, get, derived } from 'svelte/store';
import { Tween } from 'svelte/motion';
import { cubicInOut } from 'svelte/easing';
import type { ViewportSettings, AbsoluteTransform } from '../types/types';
import { currentContextId, currentViewNodes } from './ContextStore';
import { storeLogger } from '$lib/debug';

// Duplicate module instance detector
// @ts-ignore
if (!(globalThis as any).__VT_ID__) (globalThis as any).__VT_ID__ = Symbol('vt');
// eslint-disable-next-line no-console
console.info('VT id at store', (globalThis as any).__VT_ID__);
export const __VT_ID__ = (globalThis as any).__VT_ID__ as symbol;



const DEFAULT_FOCAL_TRANSFORM:  AbsoluteTransform = { x: 0, y: 0, scale: 1 };
const DEFAULT_VIEWPORT_SETTINGS: ViewportSettings = { scale: 1, posX: 0, posY: 0 };
const VIEWPORT_TWEEN_DURATION = 1000;


export const viewTransform = new Tween<ViewportSettings>(
	{ ...DEFAULT_VIEWPORT_SETTINGS },
	{ duration: VIEWPORT_TWEEN_DURATION, easing: cubicInOut }
);


export const viewportWidth = writable(0);
export const viewportHeight = writable(0);





// Viewport Actions
/** Centers the viewport on a specific canvas coordinate, maintaining current scale. */
export function centerViewOnCanvasPoint(canvasX: number, canvasY: number) {

    const viewportEl = document.getElementById('viewport');

    if (!viewportEl) {
    	console.error("[centerViewOnCanvasPoint] Viewport element not found.");
    	return;
    }

    const targetScale = 1;
    const targetPosX  = -canvasX * targetScale;
    const targetPosY  = -canvasY * targetScale;

    const newTransform = {
        scale: targetScale,
        posX: targetPosX,
        posY: targetPosY
    };

    viewTransform.set(newTransform, { duration: VIEWPORT_TWEEN_DURATION });
}





/** Centers the viewport on the current focal node. */
export function centerOnFocalNode() {

    const focalNodeId = get(currentContextId);
    const focalViewNode = get(currentViewNodes).get(focalNodeId);

    if (focalViewNode) {
    	const nodeState = focalViewNode.state.current;
    	const centerX   = nodeState.x;
    	const centerY   = nodeState.y;

    	centerViewOnCanvasPoint(centerX, centerY);

    } else {
        storeLogger.warn(`Cannot center on focal node: ViewNode ${focalNodeId} not found in current context.`);
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

    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;

    nodesInContext.forEach(viewNode => {

    	const state         = viewNode.state.current;
    	const nodeLeft      = state.x - (state.width  / 2)  * state.scale;
    	const nodeRight     = state.x + (state.width  / 2)  * state.scale;
    	const nodeTop       = state.y - (state.height / 2)  * state.scale;
    	const nodeBottom    = state.y + (state.height / 2)  * state.scale;

    	minX = Math.min(minX, nodeLeft);
    	minY = Math.min(minY, nodeTop);
    	maxX = Math.max(maxX, nodeRight);
    	maxY = Math.max(maxY, nodeBottom);
    });

    const boundsWidth   = maxX - minX;
    const boundsHeight  = maxY - minY;
    const boundsCenterX = minX + boundsWidth / 2;
    const boundsCenterY = minY + boundsHeight / 2;

    // Fallback: center on the first node found (or the focal node if it exists)
    if (boundsWidth <= 0 || boundsHeight <= 0) {
    	const firstNode = nodesInContext.values().next().value;
    	if (firstNode) {
    		const state = firstNode.state.current;
    		centerViewOnCanvasPoint(state.x + state.width / 2, state.y + state.height / 2);
    	} else {
    		viewTransform.set({ ...DEFAULT_VIEWPORT_SETTINGS }, { duration: 0 }); // Reset if truly empty
    	}
    	return;
    }

    const padding       = 0.1;
    const scaleX        = rect.width  / (boundsWidth  * (1 + padding));
    const scaleY        = rect.height / (boundsHeight * (1 + padding));

    const targetScale   = Math.min(scaleX, scaleY, 2);
    const targetPosX    = -boundsCenterX * targetScale;
    const targetPosY    = -boundsCenterY * targetScale;

    const newTransform = { scale: targetScale, posX: targetPosX, posY: targetPosY };
    viewTransform.set(newTransform, { duration: 0 });
}





/** Converts screen coordinates to canvas coordinates based on the current viewport transform. */
export function screenToCanvasCoordinates(screenX: number, screenY: number, containerRect: DOMRect): { x: number; y: number } {

    const currentTransform = viewTransform.target;
    const vw = get(viewportWidth);
    const vh = get(viewportHeight);

    // This is the inverse of the transformation applied in Viewport.svelte
    // It accounts for the pan (posX, posY), zoom (scale), and the viewport center offset (vw/2, vh/2)
    const canvasX = (screenX - containerRect.left - currentTransform.posX - vw / 2) / currentTransform.scale;
    const canvasY = (screenY - containerRect.top  - currentTransform.posY - vh / 2) / currentTransform.scale;

	return { x: canvasX, y: canvasY };
}





export { DEFAULT_FOCAL_TRANSFORM, DEFAULT_VIEWPORT_SETTINGS, VIEWPORT_TWEEN_DURATION };