<script lang="ts">
    import { nodes, currentContextId } from '$lib/karta/KartaStore';
    import type { NodeId, DataNode } from '$lib/types/types';

    let displayPath: string = '/'; // Default path

    // Reactively determine the path to display
    $: {
        const ctxId = $currentContextId;
        if (ctxId === 'global_context') {
            displayPath = '/'; // Special case for the root/global context
        } else {
            const contextNode: DataNode | undefined = $nodes.get(ctxId);
            if (contextNode && contextNode.path) {
                displayPath = contextNode.path;
            } else {
                // Fallback if node or path is somehow missing
                displayPath = '/?';
                console.warn(`Context node or path not found for ID: ${ctxId}`);
            }
        }
    }
</script>

<div class="absolute bottom-2 left-2 p-2 bg-gray-800 bg-opacity-75 text-white text-xs rounded shadow">
    Context: {displayPath}
</div>
