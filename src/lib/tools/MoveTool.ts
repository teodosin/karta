import type { Tool, NodeId } from '$lib/types/types';
import { get } from 'svelte/store';
import { viewTransform, updateNodeLayout, screenToCanvasCoordinates, contexts, currentContextId, selectedNodeIds, setSelectedNodes } from '$lib/karta/KartaStore';

export class MoveTool implements Tool {
    readonly name = 'move'; // Add the required name property
    private isDragging = false;
    // State for dragging multiple nodes
    private draggingNodeIds: Set<NodeId> | null = null;
    private initialMousePos: { canvasX: number, canvasY: number } | null = null;
    private initialNodePositions: Map<NodeId, { x: number, y: number }> | null = null;
    // Keep reference to the initially clicked element for cursor style
    private clickedNodeElement: HTMLElement | null = null;

    // Bound methods for event listeners - Use PointerEvent
    private boundHandlePointerMove: (event: PointerEvent) => void;
    private boundHandlePointerUp: (event: PointerEvent) => void;

    constructor() {
        // Bind the actual handler methods
        this.boundHandlePointerMove = this.handlePointerMove.bind(this);
        this.boundHandlePointerUp = this.handlePointerUp.bind(this);
    }

    activate() {
        // console.log('MoveTool activated'); // Keep minimal logs
        this.isDragging = false;
        this.draggingNodeIds = null;
        this.initialMousePos = null;
        this.initialNodePositions = null;
        this.clickedNodeElement = null;
        document.body.style.cursor = 'default'; // Reset cursor on activate
    }

    deactivate() {
        // console.log('MoveTool deactivated'); // Keep minimal logs
        this.removeWindowListeners();
        // Reset cursor if needed (handled below)
        // No specific node element styling to remove here anymore
        document.body.style.cursor = 'default';
        this.isDragging = false;
        this.draggingNodeIds = null;
        this.initialMousePos = null;
        this.initialNodePositions = null;
        this.clickedNodeElement = null;
    }

    // Replaces onNodeMouseDown
    onPointerDown(event: PointerEvent, targetElement: EventTarget | null): void {
        if (event.button !== 0 || !(targetElement instanceof HTMLElement)) return;

        // Check if the target is a node element
        const nodeEl = targetElement.closest('.node-wrapper') as HTMLElement | null; // Use the new wrapper class
        if (!nodeEl || !nodeEl.dataset.id) return; // Exit if not a node or no data-id

        const nodeId = nodeEl.dataset.id;
        // console.log('MoveTool onPointerDown on node', nodeId); // Keep minimal logs

        const currentSelection = get(selectedNodeIds);
        const ctxId = get(currentContextId);
        const ctxMap = get(contexts);
        const currentCtx = ctxMap.get(ctxId);

        if (!currentCtx) {
            console.error(`Current context ${ctxId} not found.`);
            return;
        }

        this.isDragging = true;
        this.clickedNodeElement = nodeEl; // Store for cursor style
        this.initialNodePositions = new Map();

        // Get mouse position in canvas coordinates
        const containerEl = nodeEl.closest('.karta-viewport-container') as HTMLElement;
        if (!containerEl) {
             console.error("Viewport container not found for coordinate conversion.");
             this.isDragging = false;
             return;
        }
        const containerRect = containerEl.getBoundingClientRect();
        const { x: mouseCanvasX, y: mouseCanvasY } = screenToCanvasCoordinates(event.clientX, event.clientY, containerRect);
        this.initialMousePos = { canvasX: mouseCanvasX, canvasY: mouseCanvasY };

        // Determine which nodes to drag and store their initial positions
        if (currentSelection.has(nodeId)) {
            // Dragging an already selected node - drag the whole group
            this.draggingNodeIds = new Set(currentSelection);
        } else {
            // Dragging an unselected node - clear selection and drag only this one
            setSelectedNodes(nodeId); // This updates the store reactively
            this.draggingNodeIds = new Set([nodeId]);
        }

        // Store initial positions for all nodes being dragged
        let allNodesFound = true;
        for (const id of this.draggingNodeIds) {
            const viewNode = currentCtx.viewNodes.get(id);
            if (viewNode) {
                this.initialNodePositions.set(id, { x: viewNode.state.current.x, y: viewNode.state.current.y });
            } else {
                console.error(`ViewNode ${id} not found in context ${ctxId} when starting drag.`);
                allNodesFound = false;
                break; // Stop if any node is missing
            }
        }

        if (!allNodesFound) {
            // Abort drag if data is inconsistent
            this.isDragging = false;
            this.draggingNodeIds = null;
            this.initialNodePositions = null;
            this.initialMousePos = null;
            this.clickedNodeElement = null;
            return;
        }

        // Add visual feedback
        document.body.style.cursor = 'grabbing';
        this.addWindowListeners();
    }

    // Replaces handleWindowMouseMove - Use PointerEvent
    private handlePointerMove(event: PointerEvent): void {
        if (!this.isDragging || !this.draggingNodeIds || !this.initialMousePos || !this.initialNodePositions) return;
        event.preventDefault();

        // Get current mouse position in canvas coordinates
        const containerEl = this.clickedNodeElement?.closest('.karta-viewport-container') as HTMLElement; // Use element ref if possible
        if (!containerEl) {
            console.error("Viewport container not found for coordinate conversion during move.");
            return; // Or try document.querySelector as fallback?
        }
        const containerRect = containerEl.getBoundingClientRect();
        const { x: mouseCanvasX, y: mouseCanvasY } = screenToCanvasCoordinates(event.clientX, event.clientY, containerRect);

        // Calculate delta from initial mouse position
        const deltaX = mouseCanvasX - this.initialMousePos.canvasX;
        const deltaY = mouseCanvasY - this.initialMousePos.canvasY;

        // Update layout for all dragged nodes
        for (const nodeId of this.draggingNodeIds) {
            const initialPos = this.initialNodePositions.get(nodeId);
            if (initialPos) {
                const newX = initialPos.x + deltaX;
                const newY = initialPos.y + deltaY;
                updateNodeLayout(nodeId, newX, newY); // This function handles store update and persistence
            }
        }
    }

    // Replaces handleWindowMouseUp - Use PointerEvent
    private handlePointerUp(event: PointerEvent): void {
        if (!this.isDragging || event.button !== 0) return;

        // Final positions are already set by handlePointerMove calling updateNodeLayout
        // Just need to clean up state

        document.body.style.cursor = 'default'; // Reset global cursor

        this.isDragging = false;
        this.draggingNodeIds = null;
        this.initialMousePos = null;
        this.initialNodePositions = null;
        this.clickedNodeElement = null;
        this.removeWindowListeners();

        // Optional: Trigger a single saveContext here if updateNodeLayout's saving is too frequent
        // const ctxId = get(currentContextId);
        // const ctx = get(contexts).get(ctxId);
        // if (ctx && localAdapter) { // Assuming localAdapter is accessible or passed
        //     localAdapter.saveContext(ctx);
        // }
    }

    // --- Removed Obsolete Methods ---
    // onCanvasClick, onCanvasMouseDown, getNodeCursorStyle, getCanvasCursorStyle
    // onWindowMouseMove, onWindowMouseUp (replaced by handlePointerMove/Up)

    // Optional: Implement updateCursor if needed for specific hover effects
    updateCursor(): void {
        if (!this.isDragging) {
            // Could set body cursor to 'grab' on hover over nodes if desired
            // document.body.style.cursor = 'grab'; // Example
        }
    }

    // --- Helper methods for listeners - Use Pointer Events ---
    private addWindowListeners() {
        // console.log("Adding window pointer listeners for MoveTool");
        window.addEventListener('pointermove', this.boundHandlePointerMove);
        window.addEventListener('pointerup', this.boundHandlePointerUp, { once: true });
    }

    private removeWindowListeners() {
        // console.log("Removing window pointer listeners for MoveTool");
        window.removeEventListener('pointermove', this.boundHandlePointerMove);
        window.removeEventListener('pointerup', this.boundHandlePointerUp);
    }
}
