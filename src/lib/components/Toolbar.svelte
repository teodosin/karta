<script lang="ts">
    import { currentTool, setTool } from '$lib/karta/KartaStore';
    import { MoveTool } from '$lib/tools/MoveTool';
    import { ConnectTool } from '$lib/tools/ConnectTool';
    import { ContextTool } from '$lib/tools/ContextTool';

    // No local setMode function needed anymore
</script>

<div class="absolute top-2 left-1/2 transform -translate-x-1/2 z-50 flex gap-2 p-2 bg-white/80 backdrop-blur-sm rounded shadow">
     <button
        class="px-3 py-1 text-sm rounded transition-colors { $currentTool instanceof MoveTool ? 'bg-indigo-600 text-white' : 'bg-gray-200 hover:bg-gray-300'}"
        on:click={() => setTool('move')}
    >
        Move
    </button>
    <button
        class="px-3 py-1 text-sm rounded transition-colors { $currentTool instanceof ConnectTool ? 'bg-indigo-600 text-white' : 'bg-gray-200 hover:bg-gray-300'}"
        on:click={() => setTool('connect')}
    >
        Connect
    </button>
     <button
        class="px-3 py-1 text-sm rounded transition-colors { $currentTool instanceof ContextTool ? 'bg-indigo-600 text-white' : 'bg-gray-200 hover:bg-gray-300'}"
        on:click={() => setTool('context')}
    >
        Context
    </button>
    <!-- Add Reset View/All later -->
</div>
