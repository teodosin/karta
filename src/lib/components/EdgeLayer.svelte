<script lang="ts">
	import {
        edges,
        layout,
        isConnecting, // Import store
        connectionSourceNodeId, // Import store
        tempLineTargetPosition // Import store
    } from '$lib/karta/KartaStore';
    import type { NodeId } from '$lib/karta/KartaStore'; // Import type
    import { get } from 'svelte/store'; // Import get

	// Reactive calculation for permanent edges
    $: edgePaths = [...$edges.values()].map(edge => {
        const sourceLayout = $layout.get(edge.source);
        const targetLayout = $layout.get(edge.target);

        if (sourceLayout && targetLayout) {
			const sourceX = sourceLayout.x + 50; // Center
			const sourceY = sourceLayout.y + 50;
			const targetX = targetLayout.x + 50; // Center
			const targetY = targetLayout.y + 50;
            return {
                id: edge.id,
                d: `M ${sourceX} ${sourceY} L ${targetX} ${targetY}`
            };
        }
        return null;
    }).filter((p): p is { id: string; d: string } => p !== null); // Type guard for filtering nulls

    // Reactive calculation for the temporary line
    $: tempLinePath = (() => {
        if (!$isConnecting) return null;

        const sourceId = $connectionSourceNodeId;
        const sourceLayout = sourceId ? $layout.get(sourceId) : null;
        const targetPos = $tempLineTargetPosition;

        if (sourceLayout && targetPos) {
            const sourceX = sourceLayout.x + 50; // Center
            const sourceY = sourceLayout.y + 50;
            return `M ${sourceX} ${sourceY} L ${targetPos.x} ${targetPos.y}`;
        }
        return null;
    })();

</script>

<svg
	class="absolute top-0 left-0 w-full h-full pointer-events-none"
	viewBox="0 0 100% 100%"
	preserveAspectRatio="none"
	style="overflow: visible;"
>
    <!-- Permanent edges -->
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