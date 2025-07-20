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
	import { selectedEdgeIds } from '$lib/karta/EdgeSelectionStore';

	// Note: Scaling edges currently not functional, do fix at some point
	export let inverseScale: number;

	$: currentCtx = $contexts.get($currentContextId);
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

	  <!-- Hit area for interaction (common for both edge types) -->
	  <path
	   id={`hit-area-${edge.id}`}
	   class="edge-hit-area"
	   d={`M ${sourceX} ${sourceY} L ${targetX} ${targetY}`}
	   data-edge-id={edge.id}
	   stroke-width={30 * inverseScale}
	  />

	  {#if edge.contains}
	   {@const dx = targetX - sourceX}
	   {@const dy = targetY - sourceY}
	   {@const length = Math.sqrt(dx * dx + dy * dy)}
	   {@const gapSize = Math.min(10, length * 0.2) * inverseScale}
	   {@const ux = dx / length}
	   {@const uy = dy / length}
	   {@const midX = sourceX + dx * 0.5}
	   {@const midY = sourceY + dy * 0.5}
	   {@const gapStartX = midX - ux * (gapSize / 2)}
	   {@const gapStartY = midY - uy * (gapSize / 2)}
	   {@const gapEndX = midX + ux * (gapSize / 2)}
	   {@const gapEndY = midY + uy * (gapSize / 2)}

	  <g class={`edge-group contains ${$selectedEdgeIds.has(edge.id) ? 'selected' : ''}`}>
	   <!-- Source side of the contains edge -->
	   <path
	   	in:fade={{ duration: 1000 }}
	   	out:fade={{ duration: 1000 }}
	   	id={`${edge.id}-source`}
	   	class="edge-part"
	   	d={`M ${sourceX} ${sourceY} L ${gapStartX} ${gapStartY}`}
	   	stroke-width={6 * inverseScale}
	   />
	   <!-- Target side of the contains edge -->
	   <path
	   	in:fade={{ duration: 1000 }}
	   	out:fade={{ duration: 1000 }}
	   	id={`${edge.id}-target`}
	   	class="edge-part"
	   	d={`M ${gapEndX} ${gapEndY} L ${targetX} ${targetY}`}
	   	stroke-width={2 * inverseScale}
	   />
	  </g>
	  {:else}
	   <!-- Standard visible edge -->
	   <path
	   	in:fade={{ duration: 1000 }}
	   	out:fade={{ duration: 1000 }}
	   	id={edge.id}
	   	class={`edge ${$selectedEdgeIds.has(edge.id) ? 'selected' : ''}`}
	   	d={`M ${sourceX} ${sourceY} L ${targetX} ${targetY}`}
	   	stroke-width={$selectedEdgeIds.has(edge.id) ? 3 * inverseScale : 2 * inverseScale}
	   />
	  {/if}
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
		stroke: #6b7280; /* gray-500 */
		/* stroke-width is now set inline based on inverseScale */
		fill: none;
		/* The transition for the hover effect (stroke color) is now handled by the browser's default transition behavior,
		which is sufficient for this simple color change. Removing the explicit 'transition' property resolves the
		conflict with Svelte's 'fade' transition, which needs to animate the 'opacity' property. */
	}
	/* Apply hover style to the visible edge when the hit area is hovered */
	:global(.edge-hit-area:hover + .edge) {
		stroke: #9ca3af; /* gray-400 */
	}
	:global(.edge.selected) {
		stroke: #3b82f6; /* blue-500 */
	}
	/* New rule for selected edge hover */
	:global(.edge-hit-area:hover + .edge.selected) {
		stroke: #93c5fd; /* blue-300 */ /* Lighter blue for selected hover */
	}

	/* --- Contains Edge Styles --- */
	:global(.edge-group.contains .edge-part) {
		stroke: rgba(156, 163, 175, 0.1); /* gray-400 at 10% opacity */
		transition: stroke 0.2s ease-in-out;
	}

	/* Hovering over the hit area affects the whole group */
	:global(.edge-hit-area:hover + .edge-group.contains .edge-part) {
		stroke: rgba(156, 163, 175, 0.4); /* gray-400 at 40% opacity */
	}

	:global(.edge-group.contains.selected .edge-part) {
		stroke: rgba(59, 130, 246, 0.5); /* blue-500 at 50% opacity */
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
