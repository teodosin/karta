// --- Karta Runtime Component ---
// This file is planned for inclusion in the MIT-licensed `karta_runtime` package.
// It defines core data structures (DataNode, ViewNode, Edge, Context, etc.).
// Essential for both editor and runtime.
// Needs to include definitions for storing "Play Mode" interaction configurations.

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
  attributes: Record<string, any>; // Holds core data (e.g., type_text), view defaults (e.g., view_isNameVisible, viewtype_fontSize), and other attributes.
  isSearchable?: boolean; // Added: Controls visibility in node search
}

// Represents the complete state needed for rendering and tweening a node's visual representation
// Note: Duplicate removed below, keeping the one with width/height
// In-memory representation of a node's view state within a context
export interface ViewNode {
  id: NodeId; // Same UUID as the corresponding DataNode
  state: Tween<TweenableNodeState>; // Tween manages core visual properties (position, size, rotation)
  // Holds context-specific attribute overrides (e.g., view_isNameVisible, viewtype_fontSize)
  // and custom user-defined attributes (unprefixed).
  attributes?: Record<string, any>;
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
    // Holds context-specific attribute overrides for storage (e.g., view_isNameVisible, viewtype_fontSize)
    // and custom user-defined attributes (unprefixed).
    attributes?: Record<string, any>;
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

// Represents the data stored for a binary asset (like an image)
export interface AssetData {
    blob: Blob;
    mimeType: string;
    name: string; // Original filename or generated name
}
// --- Import/Export Types ---

// Defines the structure for exported Karta data (JSON format)
export interface KartaExportData {
	version: number;
	exportedAt: string; // ISO timestamp string
	nodes: DataNode[];
	edges: KartaEdge[];
	contexts: StorableContext[];
	assets: {
		assetId: string;
		mimeType: string;
		name: string;
		dataUrl: string; // Asset data encoded as Data URL
	}[];
}

// Interface for defining editable properties for nodes in the Properties Panel
export interface PropertyDefinition {
  key: string; // Corresponds to a key in DataNode.attributes
  label: string; // User-friendly display name
  type: 'string' | 'number' | 'boolean' | 'textarea'; // Input control type
  // Optional fields can be added later:
  // placeholder?: string;
  // min?: number;
  // max?: number;
  // step?: number;
}

// --- End Storage-Specific Types ---

// --- Global Settings ---
export interface KartaSettings {
  version: number; // For potential future migrations
  saveLastViewedContext: boolean; // Whether to reopen the last context on startup
  // Add other global settings here as needed
}

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

// --- Available Fonts ---
export const AVAILABLE_FONTS = ['Nunito', 'Lustria', 'Bodoni Moda', 'Georgia'] as const;
export type AvailableFont = typeof AVAILABLE_FONTS[number];
