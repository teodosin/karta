import type { Tool, NodeId } from '$lib/types/types';
import { get } from 'svelte/store';
import { viewTransform, updateNodeLayout, screenToCanvasCoordinates } from '$lib/karta/KartaStore';

export class MoveTool implements Tool {
    private isDragging = false;
    private dragOffsetX = 0;
    private dragOffsetY = 0;
    private draggingNodeId: NodeId | null = null;

    // Bound methods for event listeners
    private boundHandleWindowMouseMove: (event: MouseEvent) => void;
    private boundHandleWindowMouseUp: (event: MouseEvent) => void;

    constructor() {
        // Bind the actual handler methods
        this.boundHandleWindowMouseMove = this.handleWindowMouseMove.bind(this);
        this.boundHandleWindowMouseUp = this.handleWindowMouseUp.bind(this);
    }

    activate() {
        console.log('MoveTool activated');
        this.isDragging = false; // Reset state on activation
        this.draggingNodeId = null;
    }

    deactivate() {
        console.log('MoveTool deactivated');
        this.removeWindowListeners(); // Ensure listeners are removed
    }

    onNodeMouseDown(nodeId: NodeId, event: MouseEvent, nodeElement: HTMLElement): void {
        if (event.button !== 0) return;
        console.log('MoveTool onNodeMouseDown', nodeId);

        this.isDragging = true;
        this.draggingNodeId = nodeId;

        const nodeRect = nodeElement.getBoundingClientRect();
        const currentTransform = get(viewTransform);
        this.dragOffsetX = (event.clientX - nodeRect.left) / currentTransform.scale;
        this.dragOffsetY = (event.clientY - nodeRect.top) / currentTransform.scale;

        // Add visual feedback (optional, could be CSS driven)
        nodeElement.classList.add('ring-2', 'ring-yellow-400', 'z-10');
        document.body.style.cursor = 'grabbing'; // Set global cursor

        this.addWindowListeners();
    }

    // --- Window Event Handlers (Bound) ---
    private handleWindowMouseMove(event: MouseEvent): void {
        if (!this.isDragging || !this.draggingNodeId) return;
        event.preventDefault();
        console.log('MoveTool handleWindowMouseMove');

        // Need container rect for coordinate conversion
        const containerEl = document.querySelector('.cursor-grab') as HTMLElement; // Assuming Viewport has this class
        if (!containerEl) return;
        const containerRect = containerEl.getBoundingClientRect();
        const { x: mouseCanvasX, y: mouseCanvasY } = screenToCanvasCoordinates(event.clientX, event.clientY, containerRect);

        const newX = mouseCanvasX - this.dragOffsetX;
        const newY = mouseCanvasY - this.dragOffsetY;
        updateNodeLayout(this.draggingNodeId, newX, newY);
    }

    private handleWindowMouseUp(event: MouseEvent): void {
        if (!this.isDragging || event.button !== 0) return;
        console.log('MoveTool handleWindowMouseUp');

        // Remove visual feedback (optional)
        const nodeElement = document.querySelector(`[data-id="${this.draggingNodeId}"]`) as HTMLElement;
        if (nodeElement) {
            nodeElement.classList.remove('ring-2', 'ring-yellow-400', 'z-10');
        }
        document.body.style.cursor = 'default'; // Reset global cursor

        this.isDragging = false;
        this.draggingNodeId = null;
        this.removeWindowListeners();
    }

    onCanvasClick(event: MouseEvent): void {
        console.log('MoveTool onCanvasClick', event);
    }

    onCanvasMouseDown(event: MouseEvent): void {
        console.log('MoveTool onCanvasMouseDown', event);
        // TODO: Implement canvas panning start if needed (or handle via middle mouse in Viewport)
    }

    getNodeCursorStyle(nodeId: NodeId): string {
        return 'grab';
    }

    getCanvasCursorStyle(): string {
        // Could return 'grab' if implementing canvas panning here
        return 'default';
    }

    // --- Interface Methods ---
    // These might be called if the tool needs to react to window events
    // even when not actively dragging (e.g., hover effects)
    onWindowMouseMove(event: MouseEvent): void {
        // Placeholder - could be used for hover effects if needed
    }
    onWindowMouseUp(event: MouseEvent): void {
         // Placeholder
    }


    // --- Helper methods for listeners ---
    private addWindowListeners() {
        console.log("Adding window listeners for MoveTool");
        window.addEventListener('mousemove', this.boundHandleWindowMouseMove);
        window.addEventListener('mouseup', this.boundHandleWindowMouseUp, { once: true });
    }

    private removeWindowListeners() {
        console.log("Removing window listeners for MoveTool");
        window.removeEventListener('mousemove', this.boundHandleWindowMouseMove);
        window.removeEventListener('mouseup', this.boundHandleWindowMouseUp); // Remove specifically if 'once' failed
    }
}
