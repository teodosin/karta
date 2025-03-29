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
  attributes: Record<string, any>; // Holds name, content, src, etc.
}

// Layout information for a DataNode within a specific Context
export interface ViewNode {
  id: NodeId; // Same UUID as the corresponding DataNode
  x: number;
  y: number;
  width: number; // Base width
  height: number; // Base height
  scale: number; // Uniform scaling factor (default 1)
  rotation: number; // Degrees (default 0)
}

// Represents a specific view/layout of nodes, associated with a focal node
export interface Context {
  id: NodeId; // UUID of the focal DataNode this context represents
  viewNodes: Map<NodeId, ViewNode>; // Map of NodeIDs to their layout in this context
  // TODO: Add context-specific settings like background, grid, etc. later
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
