import type { Tool, NodeId } from '$lib/types/types';
import { switchContext } from '$lib/karta/ContextStore'; // Import the action from ContextStore

export class ContextTool implements Tool {
    readonly name = 'context';
    activate() {
        // console.log('ContextTool activated');
    }
    deactivate() {
        // console.log('ContextTool deactivated');
        document.body.style.cursor = 'default'; // Reset cursor
    }

    // Implement onPointerDown to handle context switching on node click
    onPointerDown(event: PointerEvent, targetElement: EventTarget | null): void {
        if (event.button !== 0 || !(targetElement instanceof HTMLElement)) return;

        // Check if the target is a node element
        const nodeEl = targetElement.closest('.node-wrapper') as HTMLElement | null; // Use the new wrapper class
        if (!nodeEl || !nodeEl.dataset.id) return; // Exit if not a node

        const nodeId = nodeEl.dataset.id;
        console.log('ContextTool attempting switch to node', nodeId);

        // Call the KartaStore action to handle the switch
        switchContext(nodeId);
    }

    // --- Removed Obsolete Methods ---
    // onNodeMouseDown, onWindowMouseMove, onWindowMouseUp,
    // onCanvasClick, onCanvasMouseDown,
    // getNodeCursorStyle, getCanvasCursorStyle

    // Optional: Implement updateCursor if needed
    updateCursor(): void {
        document.body.style.cursor = 'context-menu'; // Example cursor
    }

    // Add other required methods from Tool interface if needed (onPointerMove, onPointerUp, etc.)
    // For now, they can be empty or omitted if not used by this tool.
    // onPointerMove(event: PointerEvent): void {}
    // onPointerUp(event: PointerEvent): void {}
    // onWheel(event: WheelEvent): void {}
    // onKeyDown(event: KeyboardEvent): void {}
    // onKeyUp(event: KeyboardEvent): void {}
}
