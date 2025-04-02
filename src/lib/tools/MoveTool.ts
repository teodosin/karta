import type { Tool, NodeId } from '$lib/types/types';
import { get } from 'svelte/store';
import { viewTransform, updateNodeLayout, screenToCanvasCoordinates } from '$lib/karta/KartaStore';

export class MoveTool implements Tool {
    readonly name = 'move'; // Add the required name property
    private isDragging = false;
    private dragOffsetX = 0;
    private dragOffsetY = 0;
    private draggingNodeId: NodeId | null = null;
    private nodeElement: HTMLElement | null = null; // Store node element reference during drag

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
        this.isDragging = false; // Reset state on activation
        this.draggingNodeId = null;
        this.nodeElement = null;
        document.body.style.cursor = 'default'; // Reset cursor on activate
    }

    deactivate() {
        // console.log('MoveTool deactivated'); // Keep minimal logs
        this.removeWindowListeners(); // Ensure listeners are removed
        if (this.nodeElement) {
            this.nodeElement.classList.remove('ring-2', 'ring-yellow-400', 'z-10');
        }
        document.body.style.cursor = 'default'; // Reset cursor on deactivate
        this.isDragging = false;
        this.draggingNodeId = null;
        this.nodeElement = null;
    }

    // Replaces onNodeMouseDown
    onPointerDown(event: PointerEvent, targetElement: EventTarget | null): void {
        if (event.button !== 0 || !(targetElement instanceof HTMLElement)) return;

        // Check if the target is a node element
        const nodeEl = targetElement.closest('.node') as HTMLElement | null;
        if (!nodeEl || !nodeEl.dataset.id) return; // Exit if not a node or no data-id

        const nodeId = nodeEl.dataset.id;
        // console.log('MoveTool onPointerDown on node', nodeId); // Keep minimal logs

        this.isDragging = true;
        this.draggingNodeId = nodeId;
        this.nodeElement = nodeEl; // Store reference

        const nodeRect = nodeEl.getBoundingClientRect();
        const currentTransform = viewTransform.target;
        this.dragOffsetX = (event.clientX - nodeRect.left) / currentTransform.scale;
        this.dragOffsetY = (event.clientY - nodeRect.top) / currentTransform.scale;

        // Add visual feedback
        nodeEl.classList.add('ring-2', 'ring-yellow-400', 'z-10');
        document.body.style.cursor = 'grabbing'; // Set global cursor

        this.addWindowListeners();
    }

    // Replaces handleWindowMouseMove - Use PointerEvent
    private handlePointerMove(event: PointerEvent): void {
        if (!this.isDragging || !this.draggingNodeId) return;
        event.preventDefault();
        // console.log('MoveTool handlePointerMove'); // Reduce console noise

        // Need container rect for coordinate conversion
        // Assuming Viewport's container has class 'karta-viewport-container' or similar
        // Let's use a more robust selector if possible, or pass container ref if needed
        const containerEl = document.querySelector('.w-full.h-screen.overflow-hidden') as HTMLElement; // Adjust selector if needed
        if (!containerEl) {
            console.error("Viewport container not found for coordinate conversion.");
            return;
        }
        const containerRect = containerEl.getBoundingClientRect();
        const { x: mouseCanvasX, y: mouseCanvasY } = screenToCanvasCoordinates(event.clientX, event.clientY, containerRect);

        const newX = mouseCanvasX - this.dragOffsetX;
        const newY = mouseCanvasY - this.dragOffsetY;
        updateNodeLayout(this.draggingNodeId, newX, newY);
    }

    // Replaces handleWindowMouseUp - Use PointerEvent
    private handlePointerUp(event: PointerEvent): void {
        if (!this.isDragging || event.button !== 0) return;
        // console.log('MoveTool handlePointerUp'); // Keep minimal logs

        // Remove visual feedback
        if (this.nodeElement) {
            this.nodeElement.classList.remove('ring-2', 'ring-yellow-400', 'z-10');
        }
        document.body.style.cursor = 'default'; // Reset global cursor

        this.isDragging = false;
        this.draggingNodeId = null;
        this.nodeElement = null;
        this.removeWindowListeners();
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
