<script lang="ts">
	import type { DataNode, ViewNode } from '$lib/types/types';
	import { currentContextId } from '$lib/karta/KartaStore';

	export let dataNode: DataNode;
	export let viewNode: ViewNode;

    let nodeElementRef: HTMLElement | null = null; // Reference to the div

</script>

{#if dataNode && viewNode}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
        bind:this={nodeElementRef}
        data-id={dataNode.id}
		class={`
			node w-[100px] h-[100px] flex items-center justify-center
			font-bold rounded absolute select-none shadow-md bg-pink-900 text-gray-100
            ${dataNode.id === $currentContextId ? 'ring-2 ring-orange-500' : ''}
		`}
		style:transform="translate({viewNode.state.current.x}px, {viewNode.state.current.y}px) scale({viewNode.state.current.scale}) rotate({viewNode.state.current.rotation}deg) translateX(-50%) translateY(-50%)"
		on:mouseenter={(e: MouseEvent) => nodeElementRef?.classList.add('shadow-lg')}
		on:mouseleave={(e: MouseEvent) => nodeElementRef?.classList.remove('shadow-lg')}
	>
		<!-- Basic Node Content - Access name from attributes -->
		{dataNode.attributes?.name ?? dataNode.ntype}
	</div>
{/if}

<style>
    .node {
        /* Add touch-action for better mobile/touchpad behavior */
        touch-action: none;
    }
</style>
