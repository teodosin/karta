<!--
// --- Karta Runtime Component ---
// This file is planned for inclusion in the MIT-licensed `karta_runtime` package.
// It is responsible for rendering edges between nodes.
// Keep this focused on display logic; avoid editor-specific features.
-->
<script lang="ts">
	import { fade } from 'svelte/transition';
	import { get } from 'svelte/store';
	import { edges } from '$lib/karta/EdgeStore';
	import { contexts, currentContextId } from '$lib/karta/ContextStore';
	import {
		isConnecting,
		connectionSourceNodeIds,
		tempLineTargetPosition,
		isReconnecting,
		reconnectingEdgeId,
		reconnectingEndpoint,
		startReconnectionProcess,
		finishReconnectionProcess,
		updateTempLinePosition
	} from '$lib/karta/ToolStore';
	import { selectedEdgeIds } from '$lib/karta/EdgeSelectionStore';
	import { screenToCanvasCoordinates } from '$lib/karta/ViewportStore';
	import type { NodeId } from '$lib/types/types';

	export let inverseScale: number;

	$: currentCtx = $contexts.get($currentContextId);
	$: reconnectingEdge = $reconnectingEdgeId ? $edges.get($reconnectingEdgeId) : null;

	function handlePointerDown(edgeId: NodeId, endpoint: 'from' | 'to', event: PointerEvent) {
		// This event is now triggered by the Viewport's master handler
		// We just need to start the reconnection process
		startReconnectionProcess(edgeId, endpoint);
		window.addEventListener('pointermove', handlePointerMove);
		window.addEventListener('pointerup', handlePointerUp, { once: true });
	}

	function handlePointerMove(event: PointerEvent) {
		if (!get(isReconnecting)) return;
		const containerEl = document.querySelector('.w-full.h-screen.overflow-hidden') as HTMLElement;
		if (!containerEl) return;
		const containerRect = containerEl.getBoundingClientRect();
		const { x, y } = screenToCanvasCoordinates(event.clientX, event.clientY, containerRect);
		updateTempLinePosition(x, y);
	}

	function handlePointerUp(event: PointerEvent) {
		if (!get(isReconnecting)) return;

		let targetNodeId: NodeId | null = null;
		let currentElement = event.target as HTMLElement;

		while (currentElement) {
			if (currentElement.dataset?.id && currentElement.classList.contains('node-wrapper')) {
				targetNodeId = currentElement.dataset.id;
				break;
			}
			const viewportContainer = document.querySelector('.w-full.h-screen.overflow-hidden');
			if (currentElement === document.body || currentElement === viewportContainer) {
				break;
			}
			currentElement = currentElement.parentElement as HTMLElement;
		}

		finishReconnectionProcess(targetNodeId);
		window.removeEventListener('pointermove', handlePointerMove);
	}
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
			{@const midX = (sourceX + targetX) / 2}
			{@const midY = (sourceY + targetY) / 2}
			{@const dx = targetX - sourceX}
			{@const dy = targetY - sourceY}
			{@const length = Math.sqrt(dx * dx + dy * dy)}
			{@const ux = length === 0 ? 0 : dx / length}
			{@const uy = length === 0 ? 0 : dy / length}

			{#if $reconnectingEdgeId !== edge.id}
				{#if edge.contains}
					{@const gapSize = Math.min(10, length * 0.2) * inverseScale}
					{@const gapStartX = midX - ux * (gapSize / 2)}
					{@const gapStartY = midY - uy * (gapSize / 2)}
					{@const gapEndX = midX + ux * (gapSize / 2)}
					{@const gapEndY = midY + uy * (gapSize / 2)}
					
					<!-- Visual Parts First -->
					<g class="edge-group contains" class:selected={$selectedEdgeIds.has(edge.id)}>
						<path in:fade={{ duration: 1000 }} out:fade={{ duration: 1000 }} id={`${edge.id}-source`} class="edge-part" d={`M ${sourceX} ${sourceY} L ${gapStartX} ${gapStartY}`} stroke-width={6 * inverseScale} />
						<path in:fade={{ duration: 1000 }} out:fade={{ duration: 1000 }} id={`${edge.id}-target`} class="edge-part" d={`M ${gapEndX} ${gapEndY} L ${targetX} ${targetY}`} stroke-width={2 * inverseScale} />
					</g>
					<!-- Hit Areas on Top -->
					<path class="edge-hit-area" data-edge-id={edge.id} data-endpoint="from" d={`M ${sourceX} ${sourceY} L ${midX} ${midY}`} stroke-width={30 * inverseScale} />
					<path class="edge-hit-area" data-edge-id={edge.id} data-endpoint="to" d={`M ${midX} ${midY} L ${targetX} ${targetY}`} stroke-width={30 * inverseScale} />

				{:else}
					{@const angle = (Math.atan2(dy, dx) * 180) / Math.PI}
					{@const markerX = sourceX + ux * (length * 0.5)}
					{@const markerY = sourceY + uy * (length * 0.5)}

					<!-- Visual Parts First -->
					<g class="edge-group" class:selected={$selectedEdgeIds.has(edge.id)}>
						<path in:fade={{ duration: 1000 }} out:fade={{ duration: 1000 }} id={edge.id} class="edge" d={`M ${sourceX} ${sourceY} L ${targetX} ${targetY}`} stroke-width={$selectedEdgeIds.has(edge.id) ? 3 * inverseScale : 2 * inverseScale} />
						<path class="edge-marker" d="M -4 -3 L 4 0 L -4 3 z" transform={`translate(${markerX}, ${markerY}) rotate(${angle}) scale(${inverseScale})`} />
					</g>
					<!-- Hit Areas on Top -->
					<path class="edge-hit-area" data-edge-id={edge.id} data-endpoint="from" d={`M ${sourceX} ${sourceY} L ${midX} ${midY}`} stroke-width={30 * inverseScale} />
					<path class="edge-hit-area" data-edge-id={edge.id} data-endpoint="to" d={`M ${midX} ${midY} L ${targetX} ${targetY}`} stroke-width={30 * inverseScale} />
				{/if}
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

	<!-- Temporary reconnection line -->
	{#if $isReconnecting && $tempLineTargetPosition && reconnectingEdge}
		{@const endpointToKeepId = $reconnectingEndpoint === 'from' ? reconnectingEdge.target : reconnectingEdge.source}
		{@const staticViewNode = currentCtx?.viewNodes.get(endpointToKeepId)}
		{@const staticState = staticViewNode?.state.current}
		{#if staticState}
			<path
				class="edge"
				d={`M ${staticState.x} ${staticState.y} L ${$tempLineTargetPosition.x} ${$tempLineTargetPosition.y}`}
				stroke-width={2 * inverseScale}
			/>
		{/if}
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
	:global(.edge.selected) {
		stroke: #3b82f6; /* blue-500 */
	}

	/* --- Contains Edge Styles --- */
	:global(.edge-group.contains .edge-part) {
		stroke: rgba(156, 163, 175, 0.1); /* gray-400 at 10% opacity */
		transition: stroke 0.2s ease-in-out;
	}
	:global(.edge-group.contains.selected .edge-part) {
		stroke: rgba(59, 130, 246, 0.5); /* blue-500 at 50% opacity */
	}

	/* --- Hit Area & Hover Effects --- */
	:global(.edge-hit-area) {
		stroke: transparent;
		fill: none;
		pointer-events: stroke;
		cursor: pointer;
	}
	:global(.edge-hit-area:hover ~ .edge-group .edge) {
		stroke: #9ca3af; /* gray-400 */
	}
	:global(.edge-hit-area:hover ~ .edge-group.selected .edge) {
		stroke: #93c5fd; /* blue-300 */
	}
	:global(.edge-hit-area:hover ~ .edge-group .edge-part) {
		stroke: rgba(156, 163, 175, 0.4); /* gray-400 at 40% opacity */
	}

	/* --- Directional Marker --- */
	:global(.edge-marker) {
		fill: #6b7280; /* gray-500 */
		stroke: none;
		transition: fill 0.2s ease-in-out;
	}
	:global(.edge-group.selected .edge-marker) {
		fill: #3b82f6; /* blue-500 */
	}
	:global(.edge-hit-area:hover ~ .edge-group .edge-marker) {
		fill: #9ca3af; /* gray-400 */
	}
	:global(.edge-hit-area:hover ~ .edge-group.selected .edge-marker) {
		fill: #93c5fd; /* blue-300 */
	}

	/* --- Temp Lines --- */
	:global(.temp-edge) {
		stroke: #86198f; /* pink-900 */
		stroke-width: 2;
		stroke-dasharray: 5, 5;
		fill: none;
	}
</style>
