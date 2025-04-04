<script lang="ts">
	import {
        edges,
        // contexts, // No longer needed directly for edge paths
        // currentContextId, // No longer needed directly for edge paths
        currentTransformTweens, // Import the tween store
        isConnecting,
        connectionSourceNodeId,
        tempLineTargetPosition
    } from '$lib/karta/KartaStore';
    import type { NodeId } from '../types/types'; // Import from types.ts
    import { get } from 'svelte/store';

	// Removed pre-calculated edgePaths. Path calculation moved into the #each loop for reactivity.

    // Reactive calculation for the temporary line based on current context
    $: tempLinePath = (() => {
        if (!$isConnecting) return null;

        const sourceId = $connectionSourceNodeId;
        const sourceTween = sourceId ? $currentTransformTweens.get(sourceId) : null;
        const targetPos = $tempLineTargetPosition;

        if (sourceTween && targetPos) {
            // Calculate center based on the tween's current dimensions and position
            const sourceState = sourceTween.current;
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
	           {@const sourceTween = $currentTransformTweens.get(edge.source)}
	           {@const targetTween = $currentTransformTweens.get(edge.target)}

	           {#if sourceTween && targetTween}
	               {@const sourceState = sourceTween.current}
	               {@const targetState = targetTween.current}
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
