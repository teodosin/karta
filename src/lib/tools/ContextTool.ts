import type { Tool, NodeId } from '$lib/types/types';

export class ContextTool implements Tool {
    activate() {
        console.log('ContextTool activated');
    }
    deactivate() {
        console.log('ContextTool deactivated');
    }
    onNodeMouseDown(nodeId: NodeId, event: MouseEvent, nodeElement: HTMLElement): void {
        console.log('ContextTool onNodeMouseDown', nodeId, event);
    }
    onWindowMouseMove(event: MouseEvent): void {
        // console.log('ContextTool onWindowMouseMove', event);
    }
    onWindowMouseUp(event: MouseEvent): void {
        console.log('ContextTool onWindowMouseUp', event);
    }
    onCanvasClick(event: MouseEvent): void {
        console.log('ContextTool onCanvasClick', event);
    }
    onCanvasMouseDown(event: MouseEvent): void {
        console.log('ContextTool onCanvasMouseDown', event);
    }
    getNodeCursorStyle(nodeId: NodeId): string {
        return 'default'; // Or maybe 'context-menu' later?
    }
    getCanvasCursorStyle(): string {
        return 'default';
    }
}
