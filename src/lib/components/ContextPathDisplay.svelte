<script lang="ts">
    import { nodes, currentContextId, historyStack, futureStack, undoContextSwitch, redoContextSwitch } from '$lib/karta/KartaStore';
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

<div class="absolute bottom-2 left-2 p-2 text-sm rounded shadow bg-gray-800 text-gray-300 flex items-center">
    <button
        type="button"
        class="text-xl hover:text-white hover:font-black hover:bg-gray-600 rounded disabled:opacity-50 disabled:font-thin p-1 px-2"
        disabled={$historyStack.length === 0}
        on:click={undoContextSwitch}
        aria-label="Undo Context Switch"
    >
        ←
    </button>
    <button
        type="button"
        class="text-xl hover:text-white hover:font-black hover:bg-gray-600 rounded disabled:opacity-50 disabled:font-thin p-1 px-2"
        disabled={$futureStack.length === 0}
        on:click={redoContextSwitch}
        aria-label="Redo Context Switch"
    >
        →
    </button>
    <span class="bg-pink-900 p-2 rounded">Context</span>
    <span class="ml-2">{displayPath}</span>
</div>
