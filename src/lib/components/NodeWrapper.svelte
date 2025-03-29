<script lang="ts">
	import { get } from 'svelte/store';
	import {
        nodes,
        layout,
        viewTransform,
        currentTool,
        updateNodeLayout,
        screenToCanvasCoordinates,
        startConnectionProcess, // Added
        updateTempLinePosition, // Added
        finishConnectionProcess // Added
    } from '$lib/karta/KartaStore';
	import type { ViewNodeLayout, NodeId } from '$lib/karta/KartaStore'; // Import NodeId

	export let nodeId: NodeId;
	export let initialLayout: ViewNodeLayout;

	// Reactive data access
	$: nodeData = $nodes.get(nodeId);
    $: nodeLayout = $layout.get(nodeId) ?? initialLayout;
    $: cursorStyle = $currentTool.getNodeCursorStyle(nodeId); // Reactive cursor style

	// State for dragging (Now managed within the tool)
	// let isDraggingNode = false; // Removed
	let nodeOffsetX = 0;
	let nodeOffsetY = 0;
    let nodeElementRef: HTMLElement | null = null; // Reference to the div


	function handleNodeMouseDown(e: MouseEvent) {
		if (e.button !== 0) return;
		e.stopPropagation();

		get(currentTool).onNodeMouseDown(nodeId, e, nodeElementRef as HTMLElement); // Pass element ref
	}

    // --- Dragging and Connecting Handlers are now inside the Tool classes ---
    // These functions are no longer needed here.

</script>

{#if nodeData && nodeLayout}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
        bind:this={nodeElementRef}
        data-id={nodeId}
		class="node w-[100px] h-[100px] bg-indigo-600 text-white flex items-center justify-center font-bold rounded absolute select-none shadow-md transition-shadow"
		style:transform="translate({nodeLayout.x}px, {nodeLayout.y}px)"
		on:mousedown={handleNodeMouseDown}
		on:mouseenter={(e) => nodeElementRef?.classList.add('shadow-lg')} 
		on:mouseleave={(e) => nodeElementRef?.classList.remove('shadow-lg')}
	>
		<!-- Basic Node Content -->
		{nodeData.label}
        ({Math.round(nodeLayout.x)}, {Math.round(nodeLayout.y)})
	</div>
{/if}

<style>
    .node {
        /* Add touch-action for better mobile/touchpad behavior */
        touch-action: none;
    }
</style>
