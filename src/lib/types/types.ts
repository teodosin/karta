export interface KartaNode {
    id: string;
    ntype: string; // Node type, e.g., 'text', 'image'
    x: number;
    y: number;
    // ... more node properties will be added later
}

export interface KartaEdge {
    id: string;
    source: string;
    target: string;
}

export type NodeId = string; // Re-export for convenience

export interface Tool {
    // Node Interactions
    onNodeMouseDown(nodeId: NodeId, event: MouseEvent, nodeElement: HTMLElement): void;
    // Note: Dragging is handled via window listeners added/removed by the tool
    // onNodeMouseMove(nodeId: NodeId, event: MouseEvent): void; // Likely not needed directly
    // onNodeMouseUp(nodeId: NodeId, event: MouseEvent): void; // Likely not needed directly

    // Window Interactions (for drags originating from nodes or canvas)
    onWindowMouseMove(event: MouseEvent): void;
    onWindowMouseUp(event: MouseEvent): void;

    // Canvas Interactions
    onCanvasClick(event: MouseEvent): void;
    onCanvasMouseDown(event: MouseEvent): void; // Renamed from DragStart for clarity
    // onCanvasDrag(event: MouseEvent): void; // Covered by onWindowMouseMove
    // onCanvasDragEnd(event: MouseEvent): void; // Covered by onWindowMouseUp

    // Lifecycle & Style
    activate(): void;
    deactivate(): void;
    getNodeCursorStyle(nodeId: NodeId): string;
    getCanvasCursorStyle(): string; // Added for canvas cursor
}
