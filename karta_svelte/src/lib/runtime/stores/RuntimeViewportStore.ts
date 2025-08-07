import { writable, get } from 'svelte/store';
import { Tween } from 'svelte/motion';
import { cubicInOut } from 'svelte/easing';
import type { ViewportSettings, AbsoluteTransform } from '../../types/types';

/**
 * RuntimeViewportStore - Viewport state management for runtime
 * 
 * This is a simplified copy of the main ViewportStore, designed for runtime
 * environments where we need viewport control but not all the editor features.
 */

export const DEFAULT_FOCAL_TRANSFORM: AbsoluteTransform = { x: 0, y: 0, scale: 1 };
export const DEFAULT_VIEWPORT_SETTINGS: ViewportSettings = { scale: 1, posX: 0, posY: 0 };
export const VIEWPORT_TWEEN_DURATION = 1000;

// Canonical tween controlling the viewport for runtime
export const runtimeViewTransform = new Tween<ViewportSettings>(
    { ...DEFAULT_VIEWPORT_SETTINGS },
    { duration: VIEWPORT_TWEEN_DURATION, easing: cubicInOut }
);

// Reactive viewport dimensions
export const runtimeViewportWidth = writable(0);
export const runtimeViewportHeight = writable(0);

// Convenience getter for the current tween value
function getCurrentTransform(): ViewportSettings {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return (runtimeViewTransform as any).current as ViewportSettings;
}

/**
 * RuntimeViewportStore class for managing viewport state
 */
export class RuntimeViewportStore {
    public readonly transform = runtimeViewTransform;
    public readonly width = runtimeViewportWidth;
    public readonly height = runtimeViewportHeight;

    /**
     * Center the viewport on a specific canvas coordinate
     */
    centerViewOnCanvasPoint(canvasX: number, canvasY: number): void {
        const current = getCurrentTransform();
        const targetScale = current.scale;
        const targetPosX = -canvasX * targetScale;
        const targetPosY = -canvasY * targetScale;
        
        runtimeViewTransform.set(
            { scale: targetScale, posX: targetPosX, posY: targetPosY },
            { duration: VIEWPORT_TWEEN_DURATION }
        );
    }

    /**
     * Set viewport dimensions
     */
    setDimensions(width: number, height: number): void {
        runtimeViewportWidth.set(width);
        runtimeViewportHeight.set(height);
    }

    /**
     * Set viewport transform directly
     */
    setTransform(settings: ViewportSettings, animated: boolean = true): void {
        const duration = animated ? VIEWPORT_TWEEN_DURATION : 0;
        runtimeViewTransform.set(settings, { duration });
    }

    /**
     * Get current transform state
     */
    getCurrentTransform(): ViewportSettings {
        return getCurrentTransform();
    }

    /**
     * Reset viewport to default state
     */
    reset(): void {
        runtimeViewTransform.set(DEFAULT_VIEWPORT_SETTINGS, { duration: VIEWPORT_TWEEN_DURATION });
    }

    /**
     * Apply zoom at a specific point
     */
    zoomAtPoint(zoomFactor: number, pointX: number, pointY: number): void {
        const current = getCurrentTransform();
        const newScale = Math.max(0.1, Math.min(10, current.scale * zoomFactor));
        
        // Calculate new position to zoom towards the point
        const newPosX = pointX - (pointX - current.posX) * (newScale / current.scale);
        const newPosY = pointY - (pointY - current.posY) * (newScale / current.scale);
        
        runtimeViewTransform.set(
            { scale: newScale, posX: newPosX, posY: newPosY },
            { duration: 200 }
        );
    }

    /**
     * Convert screen coordinates to canvas coordinates
     */
    screenToCanvas(screenX: number, screenY: number): { x: number; y: number } {
        const current = getCurrentTransform();
        return {
            x: (screenX - current.posX) / current.scale,
            y: (screenY - current.posY) / current.scale
        };
    }

    /**
     * Convert canvas coordinates to screen coordinates
     */
    canvasToScreen(canvasX: number, canvasY: number): { x: number; y: number } {
        const current = getCurrentTransform();
        return {
            x: canvasX * current.scale + current.posX,
            y: canvasY * current.scale + current.posY
        };
    }
}

// Export singleton instance for runtime use
export const runtimeViewportStore = new RuntimeViewportStore();
