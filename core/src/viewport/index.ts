import { clamp } from "../utils";
import { VIEWPORT_DEFAULTS } from "../constants";

export interface ViewportState {
  x: number;
  y: number;
  scale: number;
}

export function applyZoom(
  vp: ViewportState,
  delta: number,
  pivotX: number,
  pivotY: number
): ViewportState {
  const nextScale = clamp(
    vp.scale * (delta > 0 ? VIEWPORT_DEFAULTS.zoomStep : 1 / VIEWPORT_DEFAULTS.zoomStep),
    VIEWPORT_DEFAULTS.minScale,
    VIEWPORT_DEFAULTS.maxScale
  );

  const scaleRatio = nextScale / vp.scale;
  const nx = pivotX - (pivotX - vp.x) * scaleRatio;
  const ny = pivotY - (pivotY - vp.y) * scaleRatio;

  return { x: nx, y: ny, scale: nextScale };
}

export function screenToCanvasCoordinates(
  vp: ViewportState,
  sx: number,
  sy: number
): { x: number; y: number } {
  return {
    x: (sx - vp.x) / vp.scale,
    y: (sy - vp.y) / vp.scale
  };
}

export function canvasToScreenCoordinates(
  vp: ViewportState,
  cx: number,
  cy: number
): { x: number; y: number } {
  return {
    x: cx * vp.scale + vp.x,
    y: cy * vp.scale + vp.y
  };
}