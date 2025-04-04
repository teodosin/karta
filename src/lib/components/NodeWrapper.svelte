<script lang="ts">
    // Removed internal Tween imports, they are managed in KartaStore now
	import type { DataNode, ViewNode, TweenableNodeState } from '$lib/types/types'; // Use TweenableNodeState if needed, ViewNode is primary prop
    import { currentTransformTweens } from '$lib/karta/KartaStore'; // Import the central tween store
    import type { Tween } from 'svelte/motion'; // Still need Tween type for the variable

	// Accept DataNode and ViewNode as props
	export let dataNode: DataNode;
	export let viewNode: ViewNode;

    let nodeElementRef: HTMLElement | null = null; // Reference to the div

	// Removed handleNodeMouseDown - event bubbles up to Viewport which delegates to the tool
	// Removed reactive data access - props are already reactive
    // Removed cursor style logic - handled by Viewport/Tool

    // Get the specific tween for this node from the central store
    let transformTween: Tween<TweenableNodeState> | undefined;
    $: transformTween = $currentTransformTweens.get(viewNode.id);

    // Fallback state if tween doesn't exist immediately (should be rare)
    const fallbackState: TweenableNodeState = {
        x: viewNode.x, y: viewNode.y, scale: viewNode.scale, rotation: viewNode.rotation,
        width: viewNode.width, height: viewNode.height
    };
</script>

{#if dataNode && viewNode}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
        bind:this={nodeElementRef}
        data-id={dataNode.id}
		class="node w-[100px] h-[100px] bg-indigo-600 text-white flex items-center justify-center font-bold rounded absolute select-none shadow-md"
		style:transform="translate({transformTween?.current.x ?? fallbackState.x}px, {transformTween?.current.y ?? fallbackState.y}px) scale({transformTween?.current.scale ?? fallbackState.scale}) rotate({transformTween?.current.rotation ?? fallbackState.rotation}deg)"
		on:mouseenter={(e: MouseEvent) => nodeElementRef?.classList.add('shadow-lg')}
		on:mouseleave={(e: MouseEvent) => nodeElementRef?.classList.remove('shadow-lg')}
	>
		<!-- Basic Node Content - Access name from attributes -->
		{dataNode.attributes?.name ?? dataNode.ntype}
        ({Math.round(transformTween?.current.x ?? fallbackState.x)}, {Math.round(transformTween?.current.y ?? fallbackState.y)})
	</div>
{/if}

<style>
    .node {
        /* Add touch-action for better mobile/touchpad behavior */
        touch-action: none;
    }
</style>
