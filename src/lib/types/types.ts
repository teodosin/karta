// import type { Tween } from 'svelte/motion'; // Removed duplicate import

import type { Tween } from 'svelte/motion'; // Import Tween type

// Basic ID types (UUIDs represented as strings)
export type NodeId = string;
export type EdgeId = string;
// No separate ContextId needed, it's just a NodeId

// Core data, independent of context
export interface DataNode {
  id: NodeId; // UUID
  ntype: string; // e.g., 'text', 'image', 'context'
  createdAt: number; // Unix timestamp (ms)
  modifiedAt: number; // Unix timestamp (ms)
  path: string; // Added: Represents the node's path (simplified for client)
  attributes: Record<string, any>; // Holds name, content, src, etc.
}

// Represents the complete state needed for rendering and tweening a node's visual representation
// Note: Duplicate removed below, keeping the one with width/height
// In-memory representation of a node's view state within a context
export interface ViewNode {
  id: NodeId; // Same UUID as the corresponding DataNode
  state: Tween<TweenableNodeState>; // Tween manages all visual properties
}

// Represents an absolute transform in the canvas coordinate space
export interface AbsoluteTransform {
    x: number;
    y: number;
    scale: number;
    // Rotation removed - determined later from StorableViewNode
}

export interface TweenableNodeState {
    x: number;
    y: number;
    scale: number; // Note: This might be deprecated if we stick to width/height resizing
    rotation: number;
    width: number;  // Include dimensions needed by EdgeLayer
    height: number; // Include dimensions needed by EdgeLayer
}
// Represents a specific view/layout of nodes, associated with a focal node
export interface Context {
  id: NodeId; // UUID of the focal DataNode this context represents
  viewNodes: Map<NodeId, ViewNode>; // Map holds the new ViewNode objects containing state tweens
  viewportSettings?: ViewportSettings; // Optional saved viewport state for this context
  // TODO: Add context-specific settings like background, grid, etc. later
}

// --- Viewport ---
// NOTE: posX/posY here represent ABSOLUTE canvas coordinates for the top-left of the viewport.
// These are converted to/from relative coordinates for storage.
export interface ViewportSettings {
    scale: number;
    posX: number; // Absolute X
    posY: number; // Absolute Y
}


// --- Storage-Specific Types ---
// These represent how data is structured for persistence (e.g., in IndexedDB)

// Represents ViewNode data as stored (relative coordinates)
export interface StorableViewNode {
    id: NodeId;
    relX: number;
    relY: number;
    width: number;
    height: number;
    relScale: number;
    rotation: number;
}

// Represents ViewportSettings as stored (relative coordinates where applicable)
// Note: In current screen-relative approach, relPosX/Y store screen coords. Scale is absolute.
export interface StorableViewportSettings {
    scale: number;
    relPosX: number; // Stores target screen X
    relPosY: number; // Stores target screen Y
}

// Represents Context as stored (using StorableViewNode and StorableViewportSettings)
export interface StorableContext {
    id: NodeId;
    viewNodes: [NodeId, StorableViewNode][]; // Store as array of pairs for IndexedDB compatibility
    viewportSettings?: StorableViewportSettings;
}

// --- End Storage-Specific Types ---

// Represents connections between DataNodes (context-independent)
export interface KartaEdge {
	id: EdgeId; // Use EdgeId type
	source: NodeId; // Use NodeId type
	target: NodeId; // Use NodeId type
	// TODO: Add edge attributes or context-specific edge styles later
}

// Interface for interaction modes (Move, Connect, Context tools)
// Based on recent refactor and modern PointerEvents
export interface Tool {
	// Name of the tool (e.g., 'move', 'connect', 'context')
	name: string;

	// Called when the tool becomes active
	activate(): void;
	// Called when the tool becomes inactive
	deactivate(): void;

	// Event handlers delegated from Viewport/NodeWrapper
	// Using PointerEvent for broader input compatibility (mouse, touch, pen)
	onPointerDown?(event: PointerEvent, targetElement: EventTarget | null): void;
	onPointerMove?(event: PointerEvent): void;
	onPointerUp?(event: PointerEvent): void;
	onWheel?(event: WheelEvent): void; // WheelEvent is standard for scrolling
	onKeyDown?(event: KeyboardEvent): void;
	onKeyUp?(event: KeyboardEvent): void;
	// Add other relevant event handlers as needed (e.g., onClick, onDoubleClick)

	// Optional method to update the cursor style
	updateCursor?(): void;
}
