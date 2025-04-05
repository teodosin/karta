<script lang="ts">
    import { nodes, currentContextId } from '$lib/karta/KartaStore';
    import type { NodeId, DataNode } from '$lib/types/types';

    let displayPath: string = '/'; // Default path

    // Reactively determine the path to display
    $: {
        const ctxId = $currentContextId;
        // Removed special case for 'global_context', rely on node path attribute
        const contextNode: DataNode | undefined = $nodes.get(ctxId);
        if (contextNode && contextNode.path) {
            displayPath = contextNode.path;
        } else {
            // Fallback if node or path is somehow missing
            displayPath = '/?';
            //console.warn(`Context node or path not found for ID: ${ctxId}`);
        }
    }
</script>

<div class="absolute bottom-2 left-2 p-2 text-sm rounded shadow bg-gray-800 text-gray-300"> <!-- Reverted to default Tailwind classes -->
    Context: {displayPath}
</div>
