import type { Tool, NodeId } from '$lib/types/types';
import { get } from 'svelte/store';
import {
    startConnectionProcess,
    updateTempLinePosition,
    finishConnectionProcess,
    cancelConnectionProcess,
    isConnecting // Need to check if already connecting
} from '$lib/karta/ToolStore';
import { screenToCanvasCoordinates } from '$lib/karta/ViewportStore';

export class ConnectTool implements Tool {
    readonly name = 'connect';

    // Bound methods for event listeners - Use PointerEvent
    private boundHandlePointerMove: (event: PointerEvent) => void;
    private boundHandlePointerUp: (event: PointerEvent) => void;

    constructor() {
        this.boundHandlePointerMove = this.handlePointerMove.bind(this);
        this.boundHandlePointerUp = this.handlePointerUp.bind(this);
    }

    activate() {
        // console.log('ConnectTool activated');
        document.body.style.cursor = 'crosshair'; // Set cursor on activate
    }

    deactivate() {
        // console.log('ConnectTool deactivated');
        if (get(isConnecting)) {
            cancelConnectionProcess(); // Cancel if connection was in progress
        }
        this.removeWindowListeners();
        document.body.style.cursor = 'default'; // Reset cursor on deactivate
    }

    // Replaces onNodeMouseDown
    onPointerDown(event: PointerEvent, targetElement: EventTarget | null): void {
        if (event.button !== 0 || !(targetElement instanceof HTMLElement)) return;

        // Check if the target is a node element
        const nodeEl = targetElement.closest('.node-wrapper') as HTMLElement | null; // Use the new wrapper class
        if (!nodeEl || !nodeEl.dataset.id) return; // Exit if not a node

        const nodeId = nodeEl.dataset.id;
        // console.log('ConnectTool onPointerDown on node', nodeId); // Keep this one?
        startConnectionProcess([nodeId]); // KartaStore handles the state
        this.addWindowListeners(); // Add listeners to track pointer movement and release
    }

    // Replaces handleWindowMouseMove - Use PointerEvent
    private handlePointerMove(event: PointerEvent): void {
        if (!get(isConnecting)) return;
        // console.log('ConnectTool handlePointerMove'); // Reduce noise

        // Need container rect for coordinate conversion
        const containerEl = document.querySelector('.w-full.h-screen.overflow-hidden') as HTMLElement; // Adjust selector if needed
        if (!containerEl) {
             console.error("Viewport container not found for coordinate conversion.");
             return;
        }
        const containerRect = containerEl.getBoundingClientRect();
        const { x: mouseCanvasX, y: mouseCanvasY } = screenToCanvasCoordinates(event.clientX, event.clientY, containerRect);
        updateTempLinePosition(mouseCanvasX, mouseCanvasY);
    }

    // Replaces handleWindowMouseUp - Use PointerEvent
    private handlePointerUp(event: PointerEvent): void {
        if (!get(isConnecting) || event.button !== 0) return;
        // console.log('ConnectTool handlePointerUp'); // Keep this one?

        let targetNodeId: NodeId | null = null;
        // Use event.target for pointerup, as it reflects the element where the pointer was released
        let currentElement: HTMLElement | null = event.target as HTMLElement;

        // Traverse up DOM to find a node element with data-id
        while (currentElement) {
            if (currentElement.dataset?.id && currentElement.classList.contains('node-wrapper')) { // Use the new wrapper class
                targetNodeId = currentElement.dataset.id;
                break; // Found it
            }
            // Stop if we hit the canvas container or body
            const viewportContainer = document.querySelector('.w-full.h-screen.overflow-hidden');
            if (currentElement === document.body || currentElement === viewportContainer) {
                break;
            }
            currentElement = currentElement.parentElement;
        }

        finishConnectionProcess(targetNodeId); // KartaStore handles state reset and edge creation
        this.removeWindowListeners();
    }

    // --- Removed Obsolete Methods ---
    // onCanvasClick, onCanvasMouseDown, getNodeCursorStyle, getCanvasCursorStyle
    // onWindowMouseMove, onWindowMouseUp

    // Optional: Implement updateCursor if needed
    updateCursor(): void {
        document.body.style.cursor = 'crosshair'; // Ensure cursor stays correct
    }

    // --- Helper methods for listeners - Use Pointer Events ---
    private addWindowListeners() {
        // console.log("Adding window pointer listeners for ConnectTool");
        window.addEventListener('pointermove', this.boundHandlePointerMove);
        window.addEventListener('pointerup', this.boundHandlePointerUp, { once: true });
    }

    private removeWindowListeners() {
        // console.log("Removing window pointer listeners for ConnectTool");
        window.removeEventListener('pointermove', this.boundHandlePointerMove);
        window.removeEventListener('pointerup', this.boundHandlePointerUp);
    }
}
