import type { Tool, NodeId } from '$lib/types/types';
import { get } from 'svelte/store';
import {
    startConnectionProcess,
    updateTempLinePosition,
    finishConnectionProcess,
    cancelConnectionProcess,
    screenToCanvasCoordinates,
    isConnecting // Need to check if already connecting
} from '$lib/karta/KartaStore';

export class ConnectTool implements Tool {

    // Bound methods for event listeners
    private boundHandleWindowMouseMove: (event: MouseEvent) => void;
    private boundHandleWindowMouseUp: (event: MouseEvent) => void;

    constructor() {
        this.boundHandleWindowMouseMove = this.handleWindowMouseMove.bind(this);
        this.boundHandleWindowMouseUp = this.handleWindowMouseUp.bind(this);
    }

    activate() {
        console.log('ConnectTool activated');
    }

    deactivate() {
        console.log('ConnectTool deactivated');
        if (get(isConnecting)) {
            cancelConnectionProcess();
        }
        this.removeWindowListeners();
    }

    onNodeMouseDown(nodeId: NodeId, event: MouseEvent, nodeElement: HTMLElement): void {
        if (event.button !== 0) return;
        console.log('ConnectTool onNodeMouseDown', nodeId);
        startConnectionProcess(nodeId); // KartaStore handles the state
        this.addWindowListeners();
    }

    // --- Window Event Handlers (Bound) ---
    private handleWindowMouseMove(event: MouseEvent): void {
        if (!get(isConnecting)) return;
        console.log('ConnectTool handleWindowMouseMove');

        // Need container rect for coordinate conversion
        const containerEl = document.querySelector('.cursor-grab') as HTMLElement; // Assuming Viewport has this class
        if (!containerEl) return;
        const containerRect = containerEl.getBoundingClientRect();
        const { x: mouseCanvasX, y: mouseCanvasY } = screenToCanvasCoordinates(event.clientX, event.clientY, containerRect);
        updateTempLinePosition(mouseCanvasX, mouseCanvasY);
    }

    private handleWindowMouseUp(event: MouseEvent): void {
        if (!get(isConnecting) || event.button !== 0) return;
        console.log('ConnectTool handleWindowMouseUp');

        let targetNodeId: NodeId | null = null;
        let currentElement: HTMLElement | null = event.target as HTMLElement;

        // Traverse up DOM to find a node element with data-id
        while (currentElement) {
            if (currentElement.dataset?.id && currentElement.classList.contains('node')) {
                targetNodeId = currentElement.dataset.id;
                break; // Found it
            }
            // Stop if we hit the canvas container or body
            if (currentElement === document.body || currentElement.classList.contains('cursor-grab')) {
                break;
            }
            currentElement = currentElement.parentElement;
        }

        finishConnectionProcess(targetNodeId); // KartaStore handles state reset
        this.removeWindowListeners();
    }

    onCanvasClick(event: MouseEvent): void {
        console.log('ConnectTool onCanvasClick', event);
        if (get(isConnecting)) {
            cancelConnectionProcess();
        }
    }

    onCanvasMouseDown(event: MouseEvent): void {
        console.log('ConnectTool onCanvasMouseDown', event);
        // Don't start connection on canvas click for this tool
    }

    getNodeCursorStyle(nodeId: NodeId): string {
        return 'crosshair';
    }

    getCanvasCursorStyle(): string {
        return 'crosshair'; // Indicate connection possibility
    }

    // --- Interface Methods ---
    onWindowMouseMove(event: MouseEvent): void {
        // Handled by bound handleWindowMouseMove
    }
    onWindowMouseUp(event: MouseEvent): void {
        // Handled by bound handleWindowMouseUp
    }

    // --- Helper methods for listeners ---
    private addWindowListeners() {
        console.log("Adding window listeners for ConnectTool");
        window.addEventListener('mousemove', this.boundHandleWindowMouseMove);
        window.addEventListener('mouseup', this.boundHandleWindowMouseUp, { once: true });
    }

    private removeWindowListeners() {
        console.log("Removing window listeners for ConnectTool");
        window.removeEventListener('mousemove', this.boundHandleWindowMouseMove);
        window.removeEventListener('mouseup', this.boundHandleWindowMouseUp);
    }
}
