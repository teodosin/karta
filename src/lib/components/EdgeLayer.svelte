<!--
// --- Karta Runtime Component ---
// This file is planned for inclusion in the MIT-licensed `karta_runtime` package.
// It is responsible for rendering edges between nodes.
// Keep this focused on display logic; avoid editor-specific features.
-->
<script lang="ts">
	import { fade } from 'svelte/transition';
	import { edges } from '$lib/karta/EdgeStore';
	import { contexts, currentContextId } from '$lib/karta/ContextStore';
	import { isConnecting, connectionSourceNodeIds, tempLineTargetPosition } from '$lib/karta/ToolStore';
	import { selectedEdgeIds } from '$lib/karta/EdgeSelectionStore'; // Import the edge selection store
	// Removed: import { viewTransform } from '$lib/karta/ViewportStore';
	import type { NodeId } from '../types/types'; // Import from types.ts
	import { get } from 'svelte/store';

	// Declare inverseScale as a prop passed from Viewport
	export let inverseScale: number;

	// Get the current context object reactively
	$: currentCtx = $contexts.get($currentContextId);

	// Removed internal inverseScale calculation and log

</script>

<svg
	class="absolute top-0 left-0 w-full h-full pointer-events-none"
	style="overflow: visible;"
>

	<!-- Edges -->
	   {#each [...$edges.values()] as edge (edge.id)}
	           {@const sourceViewNode = currentCtx?.viewNodes.get(edge.source)}
	           {@const targetViewNode = currentCtx?.viewNodes.get(edge.target)}

	           {#if sourceViewNode && targetViewNode}
	               {@const sourceState = sourceViewNode.state.current}
	               {@const targetState = targetViewNode.state.current}
	               {@const sourceX = sourceState.x}
	               {@const sourceY = sourceState.y}
	               {@const targetX = targetState.x}
	               {@const targetY = targetState.y}
	               <!-- The 'each' block key is on the edge.id, so when an edge is removed from the store, this whole block is removed. -->
	               <!-- Applying the transition to the wrapping 'g' element ensures both the visible path and its hit-area fade together. -->
	               <!-- Hit area for interaction -->
	               <path
	                   id={`hit-area-${edge.id}`}
	                   class="edge-hit-area"
	                   d={`M ${sourceX} ${sourceY} L ${targetX} ${targetY}`}
	                   data-edge-id={edge.id}
	                   stroke-width={30 * inverseScale}
	               />
	               <!-- Visible edge -->
	               <path
	                   in:fade={{ duration: 1000 }}
	                   out:fade={{ duration: 1000 }}
	                   id={edge.id}
	                   class={`edge ${$selectedEdgeIds.has(edge.id) ? 'selected' : ''}`}
	                   d={`M ${sourceX} ${sourceY} L ${targetX} ${targetY}`}
	                   stroke-width={$selectedEdgeIds.has(edge.id) ? 3 * inverseScale : 2 * inverseScale}
	               />
	           {/if}
	   {/each}

	<!-- Temporary connection line(s) -->
	{#if $isConnecting && $tempLineTargetPosition}
		{#each $connectionSourceNodeIds as sourceId (sourceId)}
			{@const sourceViewNode = currentCtx?.viewNodes.get(sourceId)}
			{#if sourceViewNode}
				{@const sourceState = sourceViewNode.state.current}
				{@const sourceX = sourceState.x}
				{@const sourceY = sourceState.y}
				<path
					class="temp-edge"
					d={`M ${sourceX} ${sourceY} L ${$tempLineTargetPosition.x} ${$tempLineTargetPosition.y}`}
				/>
			{/if}
		{/each}
	{/if}
</svg>

<style>
	:global(.axis-line) {
		stroke: rgba(156, 163, 175, 0.3); /* gray-400 at 30% opacity */
		stroke-width: 1;
		fill: none;
	}
	:global(.edge) {
		stroke: #9ca3af; /* gray-400 */
		/* stroke-width is now set inline based on inverseScale */
		fill: none;
		/* The transition for the hover effect (stroke color) is now handled by the browser's default transition behavior,
		which is sufficient for this simple color change. Removing the explicit 'transition' property resolves the
		conflict with Svelte's 'fade' transition, which needs to animate the 'opacity' property. */
	}
	/* Apply hover style to the visible edge when the hit area is hovered */
	:global(.edge-hit-area:hover + .edge) {
		stroke: #d1d5db; /* gray-300 */
	}
	:global(.edge.selected) {
		stroke: #3b82f6; /* blue-500 */
	}
	/* New rule for selected edge hover */
	:global(.edge-hit-area:hover + .edge.selected) {
		stroke: #93c5fd; /* blue-300 */ /* Lighter blue for selected hover */
	}
	:global(.edge-hit-area) {
		stroke: transparent;
		/* stroke-width is now set inline based on inverseScale */
		fill: none;
		pointer-events: stroke; /* Only trigger on stroke */
		cursor: pointer; /* Indicate interactivity */
	}
	:global(.temp-edge) {
		stroke: #86198f; /* pink-900 */
		stroke-width: 2; /* Keep temp edge constant size for now */
		stroke-dasharray: 5, 5;
		fill: none;
	}
</style>
