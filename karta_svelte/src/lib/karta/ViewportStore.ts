import { writable, get } from 'svelte/store';
import { Tween } from 'svelte/motion';
import { cubicInOut } from 'svelte/easing';
import type { ViewportSettings, AbsoluteTransform } from '../types/types';
import { currentContextId, currentViewNodes } from './ContextStore';
import { storeLogger } from '$lib/debug';

const DEFAULT_FOCAL_TRANSFORM: AbsoluteTransform = { x: 0, y: 0, scale: 1 };
const DEFAULT_VIEWPORT_SETTINGS: ViewportSettings = { scale: 1, posX: 0, posY: 0 };
const VIEWPORT_TWEEN_DURATION = 1000;

// Canonical tween controlling the viewport. We use its public value as the
// single source of truth for rendering and math. Avoid mixing "target/current".
export const viewTransform = new Tween<ViewportSettings>(
	{ ...DEFAULT_VIEWPORT_SETTINGS },
	{ duration: VIEWPORT_TWEEN_DURATION, easing: cubicInOut }
);

// Reactive viewport dimensions
export const viewportWidth = writable(0);
export const viewportHeight = writable(0);

// Convenience getter for the current tween value (single source of truth)
function getCurrentTransform(): ViewportSettings {
	// Use the tween's current value as the single source of truth
	// Custom Tween exposes .current (not a Svelte readable store)
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	return (viewTransform as any).current as ViewportSettings;
}

// Viewport Actions
/** Center the viewport on a specific canvas coordinate. */
export function centerViewOnCanvasPoint(canvasX: number, canvasY: number) {
	const viewportEl = document.getElementById('viewport');
	if (!viewportEl) {
		console.error('[centerViewOnCanvasPoint] Viewport element not found.');
		return;
	}
	// Keep current scale; just re-center position
	const current = getCurrentTransform();
	const targetScale = current.scale;
	const targetPosX = -canvasX * targetScale;
	const targetPosY = -canvasY * targetScale;
	viewTransform.set({ scale: targetScale, posX: targetPosX, posY: targetPosY }, { duration: VIEWPORT_TWEEN_DURATION });
}





/** Center on the current focal node. */
export function centerOnFocalNode() {
	const focalNodeId = get(currentContextId);
	const focalViewNode = get(currentViewNodes).get(focalNodeId);
	if (focalViewNode) {
		const nodeState = focalViewNode.state.current;
		const centerX = nodeState.x;
		const centerY = nodeState.y;
		centerViewOnCanvasPoint(centerX, centerY);
	} else {
		storeLogger.warn(`Cannot center on focal node: ViewNode ${focalNodeId} not found in current context.`);
	}
}





/** Frame all nodes in current context with padding. */
export function frameContext() {
	const viewportEl = document.getElementById('viewport');
	if (!viewportEl) {
		console.error('[frameContext] Viewport element not found.');
		return;
	}
	const rect = viewportEl.getBoundingClientRect();
	const nodesInContext = get(currentViewNodes);

	let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;

	nodesInContext.forEach((viewNode) => {
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

	// Fallback: center on first node or reset
	if (boundsWidth <= 0 || boundsHeight <= 0) {
		const firstNode = nodesInContext.values().next().value;
		if (firstNode) {
			const state = firstNode.state.current;
			centerViewOnCanvasPoint(state.x + state.width / 2, state.y + state.height / 2);
		} else {
			viewTransform.set({ ...DEFAULT_VIEWPORT_SETTINGS }, { duration: 0 });
		}
		return;
	}

	const padding = 0.1;
	const scaleX = rect.width / (boundsWidth * (1 + padding));
	const scaleY = rect.height / (boundsHeight * (1 + padding));

	const targetScale = Math.min(scaleX, scaleY, 2);
	const targetPosX = -boundsCenterX * targetScale;
	const targetPosY = -boundsCenterY * targetScale;

	viewTransform.set({ scale: targetScale, posX: targetPosX, posY: targetPosY }, { duration: VIEWPORT_TWEEN_DURATION });
}





/** Convert screen coordinates to canvas coordinates using the same transform the view renders with. */
export function screenToCanvasCoordinates(
	screenX: number,
	screenY: number,
	containerRect: DOMRect
): { x: number; y: number } {
	const currentTransform = getCurrentTransform();
	const vw = get(viewportWidth);
	const vh = get(viewportHeight);

	// Inverse of the transform applied in Viewport.svelte
	const canvasX = (screenX - containerRect.left - currentTransform.posX - vw / 2) / currentTransform.scale;
	const canvasY = (screenY - containerRect.top - currentTransform.posY - vh / 2) / currentTransform.scale;

	// Debug logging
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	if (typeof window !== 'undefined' && (window as any).__KARTA_DEBUG_VIEWPORT) {
		// eslint-disable-next-line no-console
		console.debug('[ViewportStore.screenToCanvasCoordinates]', {
			screenX, screenY,
			containerLeft: containerRect.left, containerTop: containerRect.top,
			vw, vh,
			currentTransform,
			result: { x: canvasX, y: canvasY }
		});
	}

	return { x: canvasX, y: canvasY };
}





export { DEFAULT_FOCAL_TRANSFORM, DEFAULT_VIEWPORT_SETTINGS, VIEWPORT_TWEEN_DURATION };