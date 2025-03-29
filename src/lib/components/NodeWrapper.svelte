<script lang="ts">
	// Removed imports for KartaStore data, keeping only types if needed elsewhere
	import type { DataNode, ViewNode } from '$lib/types/types'; // Import new types

	// Accept DataNode and ViewNode as props
	export let dataNode: DataNode;
	export let viewNode: ViewNode;

    let nodeElementRef: HTMLElement | null = null; // Reference to the div

	// Removed handleNodeMouseDown - event bubbles up to Viewport which delegates to the tool
	// Removed reactive data access - props are already reactive
    // Removed cursor style logic - handled by Viewport/Tool
</script>

{#if dataNode && viewNode}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
        bind:this={nodeElementRef}
        data-id={dataNode.id}
		class="node w-[100px] h-[100px] bg-indigo-600 text-white flex items-center justify-center font-bold rounded absolute select-none shadow-md transition-shadow"
		style:transform="translate({viewNode.x}px, {viewNode.y}px) scale({viewNode.scale}) rotate({viewNode.rotation}deg)" 
		on:mouseenter={(e) => nodeElementRef?.classList.add('shadow-lg')}
		on:mouseleave={(e) => nodeElementRef?.classList.remove('shadow-lg')}
	>
		<!-- Basic Node Content - Access name from attributes -->
		{dataNode.attributes?.name ?? dataNode.ntype}
        ({Math.round(viewNode.x)}, {Math.round(viewNode.y)})
	</div>
{/if}

<style>
    .node {
        /* Add touch-action for better mobile/touchpad behavior */
        touch-action: none;
    }
</style>
