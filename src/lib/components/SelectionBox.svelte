<script lang="ts">
	import { derived, get } from 'svelte/store'; // Use Svelte 4 derived/get
	import { selectedNodeIds, currentViewNodes } from '$lib/karta/KartaStore'; // Removed currentViewTransform import
	import { startResize } from '$lib/interaction/ResizeLogic';
	import type { ViewNode } from '$lib/types/types'; // Removed ViewportSettings, onMount

	// Svelte 5: Use $derived rune
	// Svelte 4: Use derived function
	const selectionBounds = derived(
		[selectedNodeIds, currentViewNodes], // Depend on the stores
		([$selectedNodeIds, $currentViewNodes]) => { // Get store values in callback
			if ($selectedNodeIds.size <= 1) {
				return null;
			}

			let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
			let nodesInData: { id: string; initialViewNode: ViewNode }[] = [];

			$selectedNodeIds.forEach(id => {
				const viewNode = $currentViewNodes.get(id); // Use .get() on the Map value
				if (viewNode) {
					const state = viewNode.state.current;
					const halfWidth = state.width / 2;
					const halfHeight = state.height / 2;
					minX = Math.min(minX, state.x - halfWidth);
					minY = Math.min(minY, state.y - halfHeight);
					maxX = Math.max(maxX, state.x + halfWidth);
					maxY = Math.max(maxY, state.y + halfHeight);
					nodesInData.push({ id: id, initialViewNode: viewNode });
				}
			});

			if (nodesInData.length > 1) {
				return { minX, minY, maxX, maxY, width: maxX - minX, height: maxY - minY, nodesInData };
			}
			return null;
		}
	);

	// Debug log for selectionBounds
	$: console.log('[SelectionBox] selectionBounds:', $selectionBounds);

	// Props to receive calculated values from the parent (Viewport)
	export let inverseScale = 1;
	export let canvasOutlineWidth = 1;

	function handleResizePointerDown(event: PointerEvent, handlePosition: 'tl' | 'tr' | 'bl' | 'br') {
		// Access derived value directly
		const boundsData = get(selectionBounds); // Use get() to access store value outside reactive context
        if (!boundsData || boundsData.nodesInData.length <= 1) return;

		event.stopPropagation(); // Prevent viewport panning etc.

        // We need the ID of *one* of the nodes in the selection to pass to startResize,
        // even though the logic now uses the bounds. Let's just pick the first one.
        const primaryNodeId = boundsData.nodesInData[0].id;

		startResize(event, handlePosition, primaryNodeId, boundsData.nodesInData);
	}

</script>

{#if $selectionBounds} <!-- Use canvas bounds directly -->
	{@const bounds = $selectionBounds} <!-- Use canvas bounds -->
	<!-- Bounding Box Visual -->
	<div
		class="selection-box absolute pointer-events-none"
		style:left="{bounds.minX}px"
		style:top="{bounds.minY}px"
		style:width="{bounds.width}px"
		style:height="{bounds.height}px"
		style:border-width="{canvasOutlineWidth}px"
		aria-hidden="true"
	>
	<!-- Position/Size based on canvas coordinates -->
	<!-- Border width calculated for scale invariance -->
	<!-- DEBUG removed -->
		<!-- Resize Handles -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="resize-handle top-left"
			style="left: 0; top: 0; transform: translate(-50%, -50%) scale({inverseScale});"
			on:pointerdown={(e) => handleResizePointerDown(e, 'tl')}
		/>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="resize-handle top-right"
			style="left: 100%; top: 0; transform: translate(-50%, -50%) scale({inverseScale});"
			on:pointerdown={(e) => handleResizePointerDown(e, 'tr')}
		/>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="resize-handle bottom-left"
			style="left: 0; top: 100%; transform: translate(-50%, -50%) scale({inverseScale});"
			on:pointerdown={(e) => handleResizePointerDown(e, 'bl')}
		/>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="resize-handle bottom-right"
			style="left: 100%; top: 100%; transform: translate(-50%, -50%) scale({inverseScale});"
			on:pointerdown={(e) => handleResizePointerDown(e, 'br')}
		/>
	</div>
{/if}

<style>
	.selection-box {
		border-style: dashed;
		border-color: #60a5fa; /* Tailwind blue-400 */
		/* border-width is now set inline */
		z-index: 50; /* Above properties panel (z-40) */
	}

	/* Copied from NodeWrapper - consider abstracting */
	.resize-handle {
		position: absolute;
		width: 10px; /* Base size, visual size controlled by transform */
		height: 10px; /* Base size, visual size controlled by transform */
		background-color: #3b82f6; /* Tailwind blue-500 */
		border: 1px solid white; /* This border will also scale visually */
		border-radius: 2px;
		z-index: 51; /* Ensure handles are above selection box */
		pointer-events: auto; /* Make handles interactive */
		/* transform is now set inline */
	}
	.resize-handle.top-left { cursor: nwse-resize; }
	.resize-handle.top-right { cursor: nesw-resize; }
	.resize-handle.bottom-left { cursor: nesw-resize; }
	.resize-handle.bottom-right { cursor: nwse-resize; }
</style>