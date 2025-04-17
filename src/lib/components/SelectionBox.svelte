<script lang="ts">
	import { derived, get } from 'svelte/store'; // Use Svelte 4 derived/get
	import { selectedNodeIds, currentViewNodes, currentViewTransform } from '$lib/karta/KartaStore'; // Import readable currentViewTransform
	import { startResize } from '$lib/interaction/ResizeLogic';
	import type { ViewNode, ViewportSettings } from '$lib/types/types';
	import { onMount } from 'svelte'; // Import onMount
 	let viewportElement: HTMLElement | null = null;
 	onMount(() => {
 		// Get reference to the viewport container element after mount
 		viewportElement = document.getElementById('viewport');
 		if (!viewportElement) {
 			console.error('[SelectionBox] Viewport element (#viewport) not found!');
 		}
 	});

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

	// Svelte 5: Use $derived rune for screen coordinates
	// Svelte 4: Use derived function for screen coordinates
	const screenBounds = derived(
		[selectionBounds, currentViewTransform], // Depend on selectionBounds derived store and the readable transform store
		([$selectionBounds, $currentViewTransform]) => {
			if (!$selectionBounds || !viewportElement) return null; // Also check if viewportElement is available

			const transform = $currentViewTransform;
			const viewportRect = viewportElement.getBoundingClientRect();

			// 1. Convert canvas bounds to screen bounds (relative to window)
			const screenMinX_win = $selectionBounds.minX * transform.scale + transform.posX;
			const screenMinY_win = $selectionBounds.minY * transform.scale + transform.posY;
			const screenMaxX_win = $selectionBounds.maxX * transform.scale + transform.posX;
			const screenMaxY_win = $selectionBounds.maxY * transform.scale + transform.posY;

			// 2. Adjust to be relative to the viewport container
			const finalLeft = screenMinX_win - viewportRect.left;
			const finalTop = screenMinY_win - viewportRect.top;
			const finalWidth = screenMaxX_win - screenMinX_win;
			const finalHeight = screenMaxY_win - screenMinY_win;

			return {
				left: finalLeft,
				top: finalTop,
				width: finalWidth,
				height: finalHeight,
				nodesInData: $selectionBounds.nodesInData
			};
		}
	);

	// Debug log for screenBounds
	$: console.log('[SelectionBox] screenBounds:', $screenBounds);

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

{#if $screenBounds} <!-- Use $ prefix for auto-subscription in template -->
	{@const bounds = $screenBounds} <!-- Use $ prefix for auto-subscription -->
	<!-- Bounding Box Visual -->
	<div
		class="selection-box absolute pointer-events-none"
		style:left="{bounds.left}px"
		style:top="{bounds.top}px"
		style:width="{bounds.width}px"
		style:height="{bounds.height}px"
		aria-hidden="true"
	>
		<!-- Resize Handles -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="resize-handle top-left"
			style="left: 0; top: 0;"
			on:pointerdown={(e) => handleResizePointerDown(e, 'tl')}
		/>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="resize-handle top-right"
			style="left: 100%; top: 0;"
			on:pointerdown={(e) => handleResizePointerDown(e, 'tr')}
		/>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="resize-handle bottom-left"
			style="left: 0; top: 100%;"
			on:pointerdown={(e) => handleResizePointerDown(e, 'bl')}
		/>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="resize-handle bottom-right"
			style="left: 100%; top: 100%;"
			on:pointerdown={(e) => handleResizePointerDown(e, 'br')}
		/>
	</div>
{/if}

<style>
	.selection-box {
		border: 1px dashed #60a5fa; /* Tailwind blue-400 */
		z-index: 50; /* Above properties panel (z-40) */
	}

	/* Copied from NodeWrapper - consider abstracting */
	.resize-handle {
		position: absolute;
		width: 10px;
		height: 10px;
		background-color: #3b82f6; /* Tailwind blue-500 */
		border: 1px solid white;
		border-radius: 2px;
		z-index: 51; /* Ensure handles are above selection box */
		pointer-events: auto; /* Make handles interactive */
		transform: translate(-50%, -50%); /* Center the handle on the corner */
	}

	.resize-handle.top-left { cursor: nwse-resize; }
	.resize-handle.top-right { cursor: nesw-resize; }
	.resize-handle.bottom-left { cursor: nesw-resize; }
	.resize-handle.bottom-right { cursor: nwse-resize; }
</style>