<script lang="ts">
 import type { DataNode, ViewNode } from '$lib/types/types'; // ViewNode now contains the state tween
    // Removed imports for currentTransformTweens and internal Tween logic

	// Accept DataNode and ViewNode as props
	export let dataNode: DataNode;
	export let viewNode: ViewNode;

    let nodeElementRef: HTMLElement | null = null; // Reference to the div

	// Removed handleNodeMouseDown - event bubbles up to Viewport which delegates to the tool
	// Removed reactive data access - props are already reactive
    // Removed cursor style logic - handled by Viewport/Tool

    // The viewNode prop now contains the state tween: viewNode.state
    // No need for separate tween lookup or fallback state here,
    // assuming KartaStore correctly provides a ViewNode with a valid state Tween.
</script>

{#if dataNode && viewNode}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
        bind:this={nodeElementRef}
        data-id={dataNode.id}
		class="node w-[100px] h-[100px] bg-indigo-600 text-white flex items-center justify-center font-bold rounded absolute select-none shadow-md"
		style:transform="translate({viewNode.state.current.x}px, {viewNode.state.current.y}px) scale({viewNode.state.current.scale}) rotate({viewNode.state.current.rotation}deg)"
		on:mouseenter={(e: MouseEvent) => nodeElementRef?.classList.add('shadow-lg')}
		on:mouseleave={(e: MouseEvent) => nodeElementRef?.classList.remove('shadow-lg')}
	>
		<!-- Basic Node Content - Access name from attributes -->
		{dataNode.attributes?.name ?? dataNode.ntype}
        ({Math.round(viewNode.state.current.x)}, {Math.round(viewNode.state.current.y)})
	</div>
{/if}

<style>
    .node {
        /* Add touch-action for better mobile/touchpad behavior */
        touch-action: none;
    }
</style>
