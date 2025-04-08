<script lang="ts">
	import { get } from 'svelte/store';
	import { onMount } from 'svelte';
    import {
  viewTransform,
  screenToCanvasCoordinates,
  // createNodeAtPosition, // No longer called directly from here
  nodes, // Still need nodes for DataNode info if NodeWrapper needs it
  contexts, // Import contexts store
  currentContextId, // Import current context ID store
  currentTool,
  cancelConnectionProcess,
  // Imports for Create Node Menu
  isCreateNodeMenuOpen,
  createNodeMenuPosition,
  openCreateNodeMenu,
  closeCreateNodeMenu // Import action to close menu
 } from '$lib/karta/KartaStore';
 import NodeWrapper from './NodeWrapper.svelte';
 import EdgeLayer from './EdgeLayer.svelte';
 import CreateNodeMenu from './CreateNodeMenu.svelte'; // Import the menu component

	let canvasContainer: HTMLElement;
	let canvas: HTMLElement;

    // Reactive definition for the current context object
    $: currentCtx = $contexts.get($currentContextId);

	// State for panning
	let isPanning = false;
	let panStartX = 0;
	let panStartY = 0;
    let lastInputWasTouchpad = false; // State for heuristic

    // State for last known cursor position
    let lastScreenX = 0;
    let lastScreenY = 0;

	function handleWheel(e: WheelEvent) {
    // console.log(`handleWheel: deltaY=${e.deltaY}, deltaX=${e.deltaX}, deltaMode=${e.deltaMode}, ctrlKey=${e.ctrlKey}`); // DEBUG LOG removed
    e.preventDefault();
    if (!canvasContainer) return;

    const rect = canvasContainer.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    const currentTransform = viewTransform.target;
    const beforeZoomX = (mouseX - currentTransform.posX) / currentTransform.scale;
    const beforeZoomY = (mouseY - currentTransform.posY) / currentTransform.scale;

    let newScale = currentTransform.scale;
    let newPosX = currentTransform.posX;
    let newPosY = currentTransform.posY;
    const zoomSensitivityFactor = 0.5; // Slightly slower zoom
    const panSensitivityFactor = 1.6;  // Slightly faster pan
    const wheelZoomFactor = 1.75; // Increased Standard wheel zoom factor
    const pinchZoomSensitivity = 0.09; // Touchpad pinch zoom sensitivity

    // --- Heuristic Update ---
    // Detect if input is likely touchpad pan (both X and Y deltas present)
    const deltaThreshold = 0.1; // Use a small threshold
    if (Math.abs(e.deltaX) > deltaThreshold && Math.abs(e.deltaY) > deltaThreshold) {
        lastInputWasTouchpad = true;
    }

    if (e.ctrlKey) {
        // Pinch-to-zoom (Ctrl key pressed) - Always zoom
        const pinchFactor = 1 + pinchZoomSensitivity * zoomSensitivityFactor;
        newScale = currentTransform.scale * (e.deltaY < 0 ? pinchFactor : 1 / pinchFactor);
        newScale = Math.max(0.1, Math.min(newScale, 5));
        newPosX = mouseX - beforeZoomX * newScale;
        newPosY = mouseY - beforeZoomY * newScale;
    } else if (lastInputWasTouchpad) {
        // Touchpad panning (heuristic detected touchpad)
        newPosX = currentTransform.posX - e.deltaX * panSensitivityFactor;
        newPosY = currentTransform.posY - e.deltaY * panSensitivityFactor;
        // Keep scale the same when panning
        newScale = currentTransform.scale;
    } else {
        // Standard mouse wheel zoom (heuristic assumes mouse)
        newScale = currentTransform.scale * (e.deltaY < 0 ? wheelZoomFactor : 1 / wheelZoomFactor);
        newScale = Math.max(0.1, Math.min(newScale, 5));
        newPosX = mouseX - beforeZoomX * newScale;
        newPosY = mouseY - beforeZoomY * newScale;
    }

    viewTransform.set({ scale: newScale, posX: newPosX, posY: newPosY }, {duration: 140});

    // Call tool's wheel handler
    get(currentTool)?.onWheel?.(e);
}

	function handlePointerDown(e: PointerEvent) { // Changed to PointerEvent
		// Middle mouse panning (keep this local for now, could be a tool later)
		if (e.button === 1) {
            lastInputWasTouchpad = false; // Middle mouse click means it's definitely a mouse
			e.preventDefault();
			isPanning = true;
			const currentTransform = viewTransform.target; // Use target for initial calculation
			panStartX = e.clientX - currentTransform.posX;
			panStartY = e.clientY - currentTransform.posY;
			// Ensure canvasContainer is bound before manipulating style/listeners
			if (canvasContainer) {
			             canvasContainer.style.cursor = 'grabbing';
			             // Capture the pointer for this drag sequence
			             canvasContainer.setPointerCapture(e.pointerId);
			             // Add listeners directly to the element that captured the pointer
			             canvasContainer.addEventListener('pointermove', handleElementPointerMove);
			             canvasContainer.addEventListener('pointerup', handleElementPointerUp);
			         } else {
			              console.error("handlePointerDown: canvasContainer not bound yet!");
			         }
			return; // Don't delegate middle mouse to tool
		}

		// Delegate pointer down events to the active tool (for non-middle mouse)
        // Pass the event and the direct target element
        get(currentTool)?.onPointerDown?.(e, e.target as EventTarget | null);
	}

    // New handler for pointer move on the element during middle-mouse pan
    function handleElementPointerMove(e: PointerEvent) {
        // Check if we are still panning (redundant check if listeners are removed correctly, but safe)
        // Also check if the moving pointer is the one we captured
        // Add null check for canvasContainer
        if (isPanning && canvasContainer && canvasContainer.hasPointerCapture(e.pointerId)) {
            const newPosX = e.clientX - panStartX;
			const newPosY = e.clientY - panStartY;
            viewTransform.set({ scale: viewTransform.target.scale, posX: newPosX, posY: newPosY }, { duration: 0 });
        }
    }

    // New handler for pointer up on the element during middle-mouse pan
    function handleElementPointerUp(e: PointerEvent) {
        // Check if this is the up event for the pointer we captured and the middle button
        // Add null check for canvasContainer
        if (isPanning && e.button === 1 && canvasContainer && canvasContainer.hasPointerCapture(e.pointerId)) {
            isPanning = false;
            // No need for inner null check now, already checked above
            canvasContainer.style.cursor = 'default'; // Reset cursor
            // Remove listeners from the element
            canvasContainer.removeEventListener('pointermove', handleElementPointerMove);
            canvasContainer.removeEventListener('pointerup', handleElementPointerUp);
            // Release the pointer capture
            canvasContainer.releasePointerCapture(e.pointerId);
        }
    }

	// General pointer move on viewport (for non-panning moves, delegate to tool)
	function handleViewportPointerMove(e: PointerEvent) { // Changed to PointerEvent
        // Update last known cursor position
        lastScreenX = e.clientX;
        lastScreenY = e.clientY;

		// Delegate to the active tool
        get(currentTool)?.onPointerMove?.(e);
	}

	// General pointer up on viewport
	function handleViewportPointerUp(e: PointerEvent) { // Changed to PointerEvent
		// Delegate to the active tool
        get(currentTool)?.onPointerUp?.(e);
	}

    // Removed handleCanvasClick - click logic should be within tool's onPointerUp
function handleKeyDown(e: KeyboardEvent) {
	if (e.key === 'Tab') {
		e.preventDefault();
		if (!canvasContainer) return;

		const rect = canvasContainer.getBoundingClientRect();
		let screenX = lastScreenX;
		let screenY = lastScreenY;

		// Fallback to center if cursor hasn't moved over viewport yet
		if (screenX === 0 && screenY === 0) {
			screenX = rect.left + rect.width / 2;
			screenY = rect.top + rect.height / 2;
			console.log('Tab pressed: Cursor position unknown, using viewport center.');
		} else {
			console.log(`Tab pressed: Using cursor position (${screenX}, ${screenY})`);
		}

		// Calculate canvas coordinates
		const { x: canvasX, y: canvasY } = screenToCanvasCoordinates(screenX, screenY, rect);
		console.log(`Tab pressed: Calculated canvas coordinates (${canvasX}, ${canvasY})`);

		// Open the menu, passing both screen and canvas coordinates
		openCreateNodeMenu(screenX, screenY, canvasX, canvasY);
	}

	// Delegate keydown events to the active tool
	get(currentTool)?.onKeyDown?.(e);


        // Keep Escape key handling here for global cancel? Or move to tool?
        // Let's keep it global for now.
        if (e.key === 'Escape') {
            cancelConnectionProcess();
            closeCreateNodeMenu(); // Also close create menu on Escape
           }
          }

    function handleKeyUp(e: KeyboardEvent) {
        // Delegate keyup events to the active tool
        get(currentTool)?.onKeyUp?.(e);
	}

	function handleContextMenu(e: MouseEvent) {
		// Allow default context menu for right-click (button 2)
        // Middle mouse (button 1) default is prevented by handlePointerDown if needed
	}

	// Removed handleDoubleClick

    onMount(() => {
        // Focus the viewport container when the component mounts
        // This helps ensure keyboard events are captured correctly.
        canvasContainer?.focus();
    });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
id="viewport"
class="karta-viewport-container w-full h-screen overflow-hidden relative cursor-default bg-gray-800"
	bind:this={canvasContainer}
	on:pointerdown={handlePointerDown}
	on:pointermove={handleViewportPointerMove}
	on:pointerup={handleViewportPointerUp}
	on:wheel={handleWheel}
	on:contextmenu={handleContextMenu}
	tabindex="0"
	on:keydown={handleKeyDown}
	on:keyup={handleKeyUp}
>
	<div
		class="w-full h-full relative origin-top-left"
		bind:this={canvas}
		style:transform="translate({viewTransform.current.posX}px, {viewTransform.current.posY}px) scale({viewTransform.current.scale})"
	>
		<!-- Edge Rendering Layer -->
        <EdgeLayer />

		<!-- Node Rendering Layer - Iterate over ViewNodes in the current context -->
        {#if currentCtx}
            {#each [...currentCtx.viewNodes.values()] as viewNode (viewNode.id)}
                {@const dataNode = $nodes.get(viewNode.id)}
                {#if dataNode} <!-- Ensure corresponding DataNode exists -->
                    <NodeWrapper {viewNode} {dataNode} /> <!-- Removed nodeId prop -->
                {/if}
            {/each}
        {/if}
	</div>

	<!-- Create Node Menu (conditionally rendered) -->
	{#if $isCreateNodeMenuOpen && $createNodeMenuPosition}
		{@const transform = viewTransform.current} <!-- Access tween value directly -->
		{@const markerScreenX = $createNodeMenuPosition.canvasX * transform.scale + transform.posX}
		{@const markerScreenY = $createNodeMenuPosition.canvasY * transform.scale + transform.posY}

		<!-- Position Marker (positioned using transformed canvas coords) -->
		<div
			class="absolute w-3 h-3 border-2 border-blue-400 rounded-full bg-blue-400 bg-opacity-30 pointer-events-none z-40"
			style:left="{markerScreenX - 6}px"
			style:top="{markerScreenY - 6}px"
			aria-hidden="true"
		></div>

		<!-- The Menu Component (positioned using screen coords) -->
		<CreateNodeMenu x={$createNodeMenuPosition.screenX + 10} y={$createNodeMenuPosition.screenY + 10} />
	{/if}
</div>
