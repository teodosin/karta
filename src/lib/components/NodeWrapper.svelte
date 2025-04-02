<script lang="ts">
    import { tweened } from 'svelte/motion';
    import { cubicOut } from 'svelte/easing';
	import type { DataNode, ViewNode } from '$lib/types/types'; // Import new types

	// Accept DataNode and ViewNode as props
	export let dataNode: DataNode;
	export let viewNode: ViewNode;

    let nodeElementRef: HTMLElement | null = null; // Reference to the div

	// Removed handleNodeMouseDown - event bubbles up to Viewport which delegates to the tool
	// Removed reactive data access - props are already reactive
	   // Removed cursor style logic - handled by Viewport/Tool

	      // Tweened store for smooth transitions
	      const tweenedTransform = tweened(
	          { x: viewNode.x, y: viewNode.y, scale: viewNode.scale, rotation: viewNode.rotation },
	          { duration: 400, easing: cubicOut } // Adjust duration/easing as needed
	      );

	   // DEBUG: Log when viewNode prop changes and update tweened store
	   $: if (dataNode && viewNode) {
	       console.log(`[DEBUG NodeWrapper ${dataNode.id}] Received update. ViewNode:`, JSON.stringify(viewNode));
	          // Update the tweened store when the prop changes
	          tweenedTransform.set({ x: viewNode.x, y: viewNode.y, scale: viewNode.scale, rotation: viewNode.rotation });
	   }
</script>

{#if dataNode && viewNode}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
        bind:this={nodeElementRef}
        data-id={dataNode.id}
		class="node w-[100px] h-[100px] bg-indigo-600 text-white flex items-center justify-center font-bold rounded absolute select-none shadow-md transition-shadow"
		style:transform="translate({$tweenedTransform.x}px, {$tweenedTransform.y}px) scale({$tweenedTransform.scale}) rotate({$tweenedTransform.rotation}deg)"
		on:mouseenter={(e) => nodeElementRef?.classList.add('shadow-lg')}
		on:mouseleave={(e) => nodeElementRef?.classList.remove('shadow-lg')}
	>
		<!-- Basic Node Content - Access name from attributes -->
		{dataNode.attributes?.name ?? dataNode.ntype}
		      ({Math.round($tweenedTransform.x)}, {Math.round($tweenedTransform.y)})
	</div>
{/if}

<style>
    .node {
        /* Add touch-action for better mobile/touchpad behavior */
        touch-action: none;
    }
</style>
