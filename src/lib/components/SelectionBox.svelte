<script lang="ts">
	import { derived, get } from 'svelte/store'; // Use Svelte 4 derived/get
	import {
		selectedNodeIds
	} from '$lib/karta/SelectionStore';
	import { currentViewNodes } from '$lib/karta/ContextStore';
	import { nodes } from '$lib/karta/NodeStore'; // Import global nodes store
	import { startConnectionProcess } from '$lib/karta/ToolStore'; // Import the action to start connection
	import { startResize } from '$lib/interaction/ResizeLogic';
	import type { NodeId } from '$lib/types/types';
	import type { ViewNode } from '$lib/types/types'; // Removed ViewportSettings, onMount

	// Svelte 5: Use $derived rune
	// Svelte 4: Use derived function
	const selectionBounds = derived(
		[selectedNodeIds, currentViewNodes], // Depend on the stores
		([$selectedNodeIds, $currentViewNodes]) => { // Get store values in callback
			const size = $selectedNodeIds.size;
			if (size === 0) { // Handle empty selection
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

			// Check if any valid nodes were found (might be empty if IDs exist but nodes don't)
			if (nodesInData.length === 0) {
				return null;
			}

			// Always return bounds if at least one node is selected
			const bounds = { minX, minY, maxX, maxY, width: maxX - minX, height: maxY - minY, nodesInData };
			return bounds;
		}
	);

	// Debug log for selectionBounds

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

	function handleConnectStart(event: PointerEvent) {
		event.stopPropagation(); // Prevent viewport panning etc.
		const currentSelectedIds = get(selectedNodeIds); // Get the Set of IDs
		if (currentSelectedIds.size > 0) {
			// Check if any selected node is a ghost
			const allNodesMap = get(nodes);
			const selectedIdsArray = Array.from(currentSelectedIds);
			const containsGhost = selectedIdsArray.some(id => !allNodesMap.has(id));

			if (containsGhost) {
				// Optionally provide user feedback here (e.g., flash the handle red)
				return; // Prevent starting connection
			}

			startConnectionProcess(selectedIdsArray); // Pass IDs as an array
		}
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
	></div>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="resize-handle top-right"
		style="left: {bounds.maxX}px; top: {bounds.minY}px; transform: translate(-50%, -50%) scale({inverseScale});"
		on:pointerdown={(e) => handleResizePointerDown(e, 'tr')}
	></div>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="resize-handle bottom-left"
		style="left: {bounds.minX}px; top: {bounds.maxY}px; transform: translate(-50%, -50%) scale({inverseScale});"
		on:pointerdown={(e) => handleResizePointerDown(e, 'bl')}
	></div>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="resize-handle bottom-right"
		style="left: {bounds.maxX}px; top: {bounds.maxY}px; transform: translate(-50%, -50%) scale({inverseScale});"
		on:pointerdown={(e) => handleResizePointerDown(e, 'br')}
	></div>

	<!-- Connection Handle (Bottom Center) -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="connection-handle"
		style="left: {bounds.minX + bounds.width / 2}px; top: {bounds.maxY + 25}px; transform: translate(-50%, 100%) scale({inverseScale});"
		on:pointerdown={handleConnectStart}
	></div>
{/if}

<style>
	.selection-box {
		border-style: dashed;
		border-color: var(--color-contrast-color);
		/* border-width is now set inline */
		z-index: 50; /* Above properties panel (z-40) */
	}

	/* .handles-container style removed */

	/* Copied from NodeWrapper - consider abstracting */
	.resize-handle {
		position: absolute;
		width: 10px; /* Base size, visual size controlled by transform */
		height: 10px; /* Base size, visual size controlled by transform */
		background-color: var(--color-contrast-color);
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

	.connection-handle {
		position: absolute;
		width: 12px; /* Slightly larger? */
		height: 12px;
		background-color: #f59e0b; /* Tailwind amber-500 */
		border: 1px solid white;
		border-radius: 50%; /* Circle */
		z-index: 51; /* Above selection box, same as resize handles */
		pointer-events: auto;
		cursor: crosshair;
	}
</style>