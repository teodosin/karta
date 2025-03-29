import type { Tool, NodeId } from '$lib/types/types';

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
        const nodeEl = targetElement.closest('.node') as HTMLElement | null;
        if (!nodeEl || !nodeEl.dataset.id) return; // Exit if not a node

        const nodeId = nodeEl.dataset.id;
        console.log('ContextTool onPointerDown on node', nodeId);

        // TODO: Implement context switching logic here
        // Example: currentContextId.set(nodeId);
        alert(`Context switch to node ${nodeId} (not implemented yet)`);
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
