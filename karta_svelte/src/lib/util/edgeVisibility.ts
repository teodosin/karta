import type { EdgeVisibilityMode, KartaEdge } from '$lib/types/types';
import type { NodeId } from '$lib/types/types';

/**
 * Determines whether an edge should be visible based on filter settings and selection state
 */
export function shouldShowEdge(
    edge: KartaEdge,
    visibilityMode: EdgeVisibilityMode,
    selectedNodeIds: Set<NodeId>
): boolean {
    switch (visibilityMode) {
        case 'always':
            return true;
        case 'never':
            return false;
        case 'all-selected':
            // Show edges for all selected nodes, even if the other end isn't selected
            return selectedNodeIds.size > 0 && 
                   (selectedNodeIds.has(edge.source) || selectedNodeIds.has(edge.target));
        case 'between-selected':
            // Show if both source and target are selected, and at least one node is selected
            return selectedNodeIds.size > 0 && 
                   selectedNodeIds.has(edge.source) && 
                   selectedNodeIds.has(edge.target);
        case 'single-selected':
            // Show if exactly one node is selected and it's either the source or target
            return selectedNodeIds.size === 1 && 
                   (selectedNodeIds.has(edge.source) || selectedNodeIds.has(edge.target));
        default:
            return true;
    }
}
