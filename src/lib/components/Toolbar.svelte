<script lang="ts">
    import { currentTool, setTool } from '$lib/karta/KartaStore';
    // Tool instances are used for instanceof checks
    import { MoveTool } from '$lib/tools/MoveTool';
    import { ConnectTool } from '$lib/tools/ConnectTool';
    import { ContextTool } from '$lib/tools/ContextTool';
    // Import Lucide icons
    import { MousePointer2, Share2, Focus, type Icon as LucideIcon } from 'lucide-svelte';

    // No local setMode function needed anymore
</script>

<!-- Position left, vertical layout, dark theme styles -->
<div class="absolute top-1/2 left-2 transform -translate-y-1/2 z-50 flex flex-col gap-2 p-2 bg-gray-700/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-lg shadow-lg">
    <!-- Tool Button Structure -->
    {#each [
        { tool: 'move', label: 'Move Tool', icon: MousePointer2, instance: MoveTool },
        { tool: 'connect', label: 'Connect Tool', icon: Share2, instance: ConnectTool },
        { tool: 'context', label: 'Context Tool', icon: Focus, instance: ContextTool }
      ] as item (item.tool)}
        <div class="relative group"> <!-- Container for tooltip positioning -->
            <button
                type="button"
                class="p-2 rounded transition-colors focus:outline-none focus:ring-2 focus:ring-indigo-400
                       { $currentTool instanceof item.instance
                           ? 'bg-indigo-600 text-white'
                           : 'text-gray-300 hover:bg-gray-600 hover:text-white'}"
                on:click={() => setTool(item.tool as 'move' | 'connect' | 'context')}
                aria-label={item.label}
            >
                <svelte:component
                    this={item.icon}
                    class="transition-all"
                    strokeWidth={$currentTool instanceof item.instance ? 2.5 : 1.5}
                    size={20}
                />
                 <!-- Tooltip shown on hover -->
                 <span class="absolute left-full top-1/2 transform -translate-y-1/2 ml-3 px-2 py-1 bg-gray-900 text-white text-xs rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none">
                    {item.label}
                </span>
            </button>
        </div>
    {/each}
    <!-- TODO: Add Reset View/All later -->
</div>
