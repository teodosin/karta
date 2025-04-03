<script lang="ts">
	import {
        edges,
        contexts, // Use contexts store
        currentContextId, // Need current context ID
        isConnecting,
        connectionSourceNodeId,
        tempLineTargetPosition
    } from '$lib/karta/KartaStore';
    import type { NodeId } from '../types/types'; // Import from types.ts
    import { get } from 'svelte/store';

	// Reactive calculation for permanent edges based on current context
    $: currentCtx = $contexts.get($currentContextId);
    // Removed temporary debug log

    $: edgePaths = [...$edges.values()].map(edge => {
        // Check if BOTH source and target nodes exist in the current context's viewNodes
        const sourceViewNode = currentCtx?.viewNodes.get(edge.source);
        const targetViewNode = currentCtx?.viewNodes.get(edge.target);

        // Only proceed if both ViewNodes are found in the current context
        if (sourceViewNode && targetViewNode) {
			// Calculate center based on ViewNode dimensions
			const sourceX = sourceViewNode.x + sourceViewNode.width / 2;
			const sourceY = sourceViewNode.y + sourceViewNode.height / 2;
			const targetX = targetViewNode.x + targetViewNode.width / 2;
			const targetY = targetViewNode.y + targetViewNode.height / 2;
            return {
                id: edge.id,
                d: `M ${sourceX} ${sourceY} L ${targetX} ${targetY}`
            };
        }
        return null;
    }).filter((p): p is { id: string; d: string } => p !== null); // Type guard for filtering nulls

    // Reactive calculation for the temporary line based on current context
    $: tempLinePath = (() => {
        if (!$isConnecting) return null;

        const sourceId = $connectionSourceNodeId;
        const sourceViewNode = sourceId ? currentCtx?.viewNodes.get(sourceId) : null;
        const targetPos = $tempLineTargetPosition;

        if (sourceViewNode && targetPos) {
            // Calculate center based on ViewNode dimensions
            const sourceX = sourceViewNode.x + sourceViewNode.width / 2;
            const sourceY = sourceViewNode.y + sourceViewNode.height / 2;
            return `M ${sourceX} ${sourceY} L ${targetPos.x} ${targetPos.y}`;
        }
        return null;
    })();

</script>

<svg
	class="absolute top-0 left-0 w-full h-full pointer-events-none"
	style="overflow: visible;"
>
	<!-- Origin Axes -->
	<line x1="0" y1="-1000000" x2="0" y2="1000000" class="axis-line" /> <!-- Y Axis -->
	<line x1="-1000000" y1="0" x2="1000000" y2="0" class="axis-line" /> <!-- X Axis -->

	<!-- Edges -->
	   {#each edgePaths as pathData (pathData.id)}
        <path
            id={pathData.id}
            class="edge"
            d={pathData.d}
        />
    {/each}

	<!-- Temporary connection line -->
    {#if tempLinePath}
        <path
            class="temp-edge"
            d={tempLinePath}
        />
    {/if}
</svg>

<style>
	:global(.axis-line) {
		stroke: rgba(0, 0, 0, 0.15); /* Darker gray, semi-transparent */
		stroke-width: 1;
		vector-effect: non-scaling-stroke; /* Keep width constant on zoom */
		fill: none;
	}
	:global(.edge) {
		stroke: #9ca3af; /* gray-400 */
		stroke-width: 2;
		fill: none;
	}
    :global(.temp-edge) {
        stroke: #3b82f6; /* blue-500 */
        stroke-width: 2;
        stroke-dasharray: 5, 5;
        fill: none;
    }
</style>
