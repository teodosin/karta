<!--
// --- Karta Runtime Component ---
// This file is planned for inclusion in the MIT-licensed `karta_runtime` package.
// It is responsible for rendering edges between nodes.
// Keep this focused on display logic; avoid editor-specific features.
-->
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
            const sourceX = sourceState.x; // Use center X directly
            const sourceY = sourceState.y; // Use center Y directly
            return `M ${sourceX} ${sourceY} L ${targetPos.x} ${targetPos.y}`;
        }
        return null;
    })();

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
