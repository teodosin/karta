<!--
// --- Karta Runtime Component ---
// This file is planned for inclusion in the MIT-licensed `karta_runtime` package.
// It is responsible for rendering edges between nodes.
// Keep this focused on display logic; avoid editor-specific features.
-->
<script lang="ts">
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
	               {@const sourceX = sourceState.x} // Use center X directly
	               {@const sourceY = sourceState.y} // Use center Y directly
	               {@const targetX = targetState.x} // Use center X directly
	               {@const targetY = targetState.y} // Use center Y directly
	               <!-- Hit area for interaction -->
	               <!-- Uses stroke-width scaled by inverseScale to maintain constant screen size, mimicking SelectionBox handle scaling -->
	               <path
	                   id={`hit-area-${edge.id}`}
	                   class="edge-hit-area"
	                   d={`M ${sourceX} ${sourceY} L ${targetX} ${targetY}`}
	                   data-edge-id={edge.id}
	       stroke-width={30 * inverseScale}
	               />
	               <!-- Visible edge -->
	               <!-- Uses stroke-width scaled by inverseScale to maintain constant screen size -->
	               <path
	                   id={edge.id}
	                   class={`edge ${$selectedEdgeIds.has(edge.id) ? 'selected' : ''}`}
	                   d={`M ${sourceX} ${sourceY} L ${targetX} ${targetY}`}
	       stroke-width={2 * inverseScale}
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
		/* Add transition for hover effect */
		transition: stroke 0.1s ease-in-out;
	}
	:global(.edge:hover) {
		stroke: #d1d5db; /* gray-300 */
	}
	:global(.edge.selected) {
		stroke: #3b82f6; /* blue-500 */
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
