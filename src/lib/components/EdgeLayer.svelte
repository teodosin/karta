<script lang="ts">
	import {
        edges,
        contexts, // Need contexts store again
        currentContextId, // Need current context ID again
        // Removed currentTransformTweens import
        isConnecting,
        connectionSourceNodeId,
        tempLineTargetPosition
    } from '$lib/karta/KartaStore';
    import type { NodeId } from '../types/types'; // Import from types.ts
    import { get } from 'svelte/store';

	// Get the current context object reactively
	   $: currentCtx = $contexts.get($currentContextId);

    // Reactive calculation for the temporary line
    $: tempLinePath = (() => {
        if (!$isConnecting) return null;

        const sourceId = $connectionSourceNodeId;
        // Get ViewNode from the current context
        const sourceViewNode = sourceId ? currentCtx?.viewNodes.get(sourceId) : null;
        const targetPos = $tempLineTargetPosition;

        if (sourceViewNode && targetPos) {
            // Calculate center based on the ViewNode's state tween
            const sourceState = sourceViewNode.state.current;
            const sourceX = sourceState.x + sourceState.width / 2;
            const sourceY = sourceState.y + sourceState.height / 2;
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
	   {#each [...$edges.values()] as edge (edge.id)}
	           {@const sourceViewNode = currentCtx?.viewNodes.get(edge.source)}
	           {@const targetViewNode = currentCtx?.viewNodes.get(edge.target)}

	           {#if sourceViewNode && targetViewNode}
	               {@const sourceState = sourceViewNode.state.current}
	               {@const targetState = targetViewNode.state.current}
	               {@const sourceX = sourceState.x + sourceState.width / 2}
	               {@const sourceY = sourceState.y + sourceState.height / 2}
	               {@const targetX = targetState.x + targetState.width / 2}
	               {@const targetY = targetState.y + targetState.height / 2}
	               <path
	                   id={edge.id}
	                   class="edge"
	                   d={`M ${sourceX} ${sourceY} L ${targetX} ${targetY}`}
	               />
	           {/if}
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
		stroke: rgba(156, 163, 175, 0.3); /* gray-400 at 30% opacity */
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
        stroke: #86198f; /* pink-900 */
        stroke-width: 2;
        stroke-dasharray: 5, 5;
        fill: none;
    }
</style>
