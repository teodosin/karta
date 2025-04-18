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
			console.log('[SelectionBox Derived] Running...'); // DEBUG
			console.log('[SelectionBox Derived] Selected IDs:', $selectedNodeIds); // DEBUG
			console.log('[SelectionBox Derived] Current View Nodes:', $currentViewNodes); // DEBUG
			const size = $selectedNodeIds.size;
			if (size === 0) { // Handle empty selection
				console.log('[SelectionBox Derived] Size is 0, returning null.'); // DEBUG
				return null;
			}

			let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
			let nodesInData: { id: string; initialViewNode: ViewNode }[] = [];

			$selectedNodeIds.forEach(id => {
				console.log(`[SelectionBox Derived] Checking ID: ${id}`); // DEBUG
				const viewNode = $currentViewNodes.get(id); // Use .get() on the Map value
				console.log(`[SelectionBox Derived] Found viewNode for ${id}:`, viewNode); // DEBUG
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

			// Check if any valid nodes were found (might be empty if IDs exist but nodes don't)
			if (nodesInData.length === 0) {
				console.log('[SelectionBox Derived] nodesInData is empty, returning null.'); // DEBUG
				return null;
			}

			// Always return bounds if at least one node is selected
			const bounds = { minX, minY, maxX, maxY, width: maxX - minX, height: maxY - minY, nodesInData };
			console.log('[SelectionBox Derived] Returning bounds:', bounds); // DEBUG
			return bounds;
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
		// Only check if boundsData exists (meaning at least one node is selected)
		if (!boundsData) return;

		event.stopPropagation(); // Prevent viewport panning etc.

        // We need the ID of *one* of the nodes in the selection to pass to startResize,
        // even though the logic now uses the bounds. Let's just pick the first one.
        const primaryNodeId = boundsData.nodesInData[0].id;

		startResize(event, handlePosition, primaryNodeId, boundsData.nodesInData);
	}

</script>

{#if $selectionBounds}
	{@const bounds = $selectionBounds}
	{@const numSelected = $selectedNodeIds.size} <!-- Get size for conditional rendering -->

	<!-- Bounding Box Visual (Only for multi-select) -->
	{#if numSelected > 1} <!-- Reverted: Dashed box only for multi-select -->
		<div
			class="selection-box absolute pointer-events-none"
			style:left="{bounds.minX}px"
			style:top="{bounds.minY}px"
			style:width="{bounds.width}px"
			style:height="{bounds.height}px"
			style:border-width="{canvasOutlineWidth}px"
			aria-hidden="true"
		>
		</div>
	{/if}

	<!-- Resize Handles (For single and multi-select) - Positioned directly -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="resize-handle top-left"
		style="left: {bounds.minX}px; top: {bounds.minY}px; transform: translate(-50%, -50%) scale({inverseScale});"
		on:pointerdown={(e) => handleResizePointerDown(e, 'tl')}
	/>
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="resize-handle top-right"
		style="left: {bounds.maxX}px; top: {bounds.minY}px; transform: translate(-50%, -50%) scale({inverseScale});"
		on:pointerdown={(e) => handleResizePointerDown(e, 'tr')}
	/>
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="resize-handle bottom-left"
		style="left: {bounds.minX}px; top: {bounds.maxY}px; transform: translate(-50%, -50%) scale({inverseScale});"
		on:pointerdown={(e) => handleResizePointerDown(e, 'bl')}
	/>
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="resize-handle bottom-right"
		style="left: {bounds.maxX}px; top: {bounds.maxY}px; transform: translate(-50%, -50%) scale({inverseScale});"
		on:pointerdown={(e) => handleResizePointerDown(e, 'br')}
	/>
{/if}

<style>
	.selection-box {
		border-style: dashed;
		border-color: #60a5fa; /* Tailwind blue-400 */
		/* border-width is now set inline */
		z-index: 50; /* Above properties panel (z-40) */
	}

	/* .handles-container style removed */

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