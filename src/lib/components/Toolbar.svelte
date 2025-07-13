<script lang="ts">
	import { settings } from '$lib/karta/SettingsStore';
    import { currentTool, setTool } from '$lib/karta/ToolStore';
    import { saveCurrentContext } from '$lib/karta/ContextStore';
    // Tool instances are used for instanceof checks
    import { MoveTool } from '$lib/tools/MoveTool';
    import { ConnectTool } from '$lib/tools/ConnectTool';
    import { ContextTool } from '$lib/tools/ContextTool';
    // Import Lucide icons
    import { MousePointer2, Workflow, Focus, Save, type Icon as LucideIcon } from 'lucide-svelte';

    // No local setMode function needed anymore
</script>

<!-- Position left, vertical layout, dark theme styles -->
<div class="absolute top-1/2 left-2 transform -translate-y-1/2 z-50 flex flex-col gap-2 p-2 backdrop-blur-sm rounded-lg shadow-lg bg-panel-bg">
    <!-- Tool Button Structure -->
    {#each [
        { tool: 'move', label: 'Move Tool', icon: MousePointer2, instance: MoveTool },
        { tool: 'connect', label: 'Connect Tool', icon: Workflow, instance: ConnectTool },
        { tool: 'context', label: 'Context Tool', icon: Focus, instance: ContextTool } // Removed trailing comma
      ] as item (item.tool)}
        {#if item.tool !== 'connect'} <!-- Temporarily hide connect tool button -->
        <div class="relative group"> <!-- Container for tooltip positioning -->
            <button
                type="button"
                class="toolbar-button p-2 rounded transition-colors focus:outline-none focus:ring-2"
                style="--panel-hl: {$settings.colorTheme['panel-hl']}; { $currentTool instanceof item.instance ? `background-color: ${$settings.colorTheme['panel-hl']};` : '' }"
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
                 <span class="absolute left-full top-1/2 transform -translate-y-1/2 ml-3 px-2 py-1 text-xs rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none bg-gray-900 text-white">
                    {item.label}
                </span>
            </button>
        </div> <!-- End of the div.relative.group -->
        {/if} <!-- End of the if block -->
    {/each}

    <!-- Divider -->
    <div class="h-px bg-gray-600/50 w-full my-1"></div>

    <!-- Save Button -->
    <div class="relative group">
        <button
            type="button"
            class="toolbar-button p-2 rounded transition-colors focus:outline-none focus:ring-2"
            style="--panel-hl: {$settings.colorTheme['panel-hl']};"
            on:click={saveCurrentContext}
            aria-label="Save Layout"
        >
            <Save class="transition-all" strokeWidth={1.5} size={20} />
            <!-- Tooltip shown on hover -->
            <span
                class="absolute left-full top-1/2 transform -translate-y-1/2 ml-3 px-2 py-1 text-xs rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none bg-gray-900 text-white"
            >
                Save Layout
            </span>
        </button>
    </div>

    <!-- TODO: Add Reset View/All later -->
</div>
