<script lang="ts">
	import { get } from 'svelte/store';
    import {
        viewTransform,
        screenToCanvasCoordinates,
        createNodeAtPosition,
        nodes, // Still need nodes for DataNode info if NodeWrapper needs it
        contexts, // Import contexts store
        currentContextId, // Import current context ID store
        currentTool,
        cancelConnectionProcess
    } from '$lib/karta/KartaStore';
	import NodeWrapper from './NodeWrapper.svelte';
    import EdgeLayer from './EdgeLayer.svelte';

	let canvasContainer: HTMLElement;
	let canvas: HTMLElement;

    // Reactive definition for the current context object
    $: currentCtx = $contexts.get($currentContextId);

	// State for panning
	let isPanning = false;
	let panStartX = 0;
	let panStartY = 0;

	function handleWheel(e: WheelEvent) {
    e.preventDefault();
    if (!canvasContainer) return;

    const rect = canvasContainer.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    const currentTransform = get(viewTransform);
    const beforeZoomX = (mouseX - currentTransform.posX) / currentTransform.scale;
    const beforeZoomY = (mouseY - currentTransform.posY) / currentTransform.scale;

    let newScale = currentTransform.scale;
    let newPosX = currentTransform.posX;
    let newPosY = currentTransform.posY;
    const zoomSensitivityFactor = 2.0; // Slightly slower zoom
    const panSensitivityFactor = 4.5;  // Slightly faster pan

    if (e.ctrlKey) {
        // Pinch-to-zoom (Ctrl + Wheel)
        newScale = currentTransform.scale * (e.deltaY < 0 ? 1 + 0.1 * zoomSensitivityFactor : 1 / (1 + 0.1 * zoomSensitivityFactor)); // Slower zoom factor
        newScale = Math.max(0.1, Math.min(newScale, 5));
        newPosX = mouseX - beforeZoomX * newScale;
        newPosY = mouseY - beforeZoomY * newScale;
    } else if (e.deltaMode === 0) {
        // Touchpad panning (Pixel deltaMode)
        newPosX = currentTransform.posX - e.deltaX * panSensitivityFactor; // Faster panning speed
        newPosY = currentTransform.posY - e.deltaY * panSensitivityFactor; // Faster panning speed
    } else {
        // Mouse wheel zoom (Line or Page deltaMode)
        newScale = currentTransform.scale * (e.deltaY < 0 ? 1.1 : 1 / 1.1);
        newScale = Math.max(0.1, Math.min(newScale, 5));
        newPosX = mouseX - beforeZoomX * newScale;
        newPosY = mouseY - beforeZoomY * newScale;
    }

    viewTransform.set({ scale: newScale, posX: newPosX, posY: newPosY });

    // Call tool's wheel handler
    get(currentTool)?.onWheel?.(e);
}

	function handlePointerDown(e: PointerEvent) { // Changed to PointerEvent
		// Middle mouse panning (keep this local for now, could be a tool later)
		if (e.button === 1) {
			e.preventDefault();
			isPanning = true;
			const currentTransform = get(viewTransform);
			panStartX = e.clientX - currentTransform.posX;
			panStartY = e.clientY - currentTransform.posY;
			if(canvasContainer) canvasContainer.style.cursor = 'grabbing';
			// Add window listeners for middle-mouse panning
			window.addEventListener('mousemove', handleMiddleMouseMove);
			window.addEventListener('pointerup', handleMiddleMouseUp, { once: true }); // Changed to pointerup
			return; // Don't delegate middle mouse to tool
		}

		// Delegate pointer down events to the active tool
        // Pass the event and the direct target element
        get(currentTool)?.onPointerDown?.(e, e.target as EventTarget | null);
	}

	// Renamed from handleMouseMove to avoid conflict
	function handleMiddleMouseMove(e: MouseEvent) { // Changed back to MouseEvent for window listener
		if (isPanning) {
			const newPosX = e.clientX - panStartX;
			const newPosY = e.clientY - panStartY;
             viewTransform.set({ scale: $viewTransform.scale, posX: newPosX, posY: newPosY }, { duration: 0 });
		}
	}

	// Renamed from handleMouseUp to avoid conflict
	function handleMiddleMouseUp(e: PointerEvent) { // Changed to PointerEvent
		if (isPanning && e.button === 1) {
			isPanning = false;
			// Cursor is now handled reactively by the tool
			// Remove window listeners for middle-mouse panning
			window.removeEventListener('pointermove', handleMiddleMouseMove); // Changed to pointermove
			// pointerup listener removed by 'once: true'
		}
	}

	// General pointer move on viewport
	function handleViewportPointerMove(e: PointerEvent) { // Changed to PointerEvent
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
            // Create at view center for now
            const rect = canvasContainer.getBoundingClientRect();
            const screenX = rect.left + rect.width / 2;
            const screenY = rect.top + rect.height / 2;
            const {x, y} = screenToCanvasCoordinates(screenX, screenY, rect);
            createNodeAtPosition(x, y, 'text'); // Use lowercase ntype
        }
        // Delegate keydown events to the active tool
        get(currentTool)?.onKeyDown?.(e);

        // Keep Escape key handling here for global cancel? Or move to tool?
        // Let's keep it global for now.
        if (e.key === 'Escape') {
            cancelConnectionProcess();
        }
    }

    function handleKeyUp(e: KeyboardEvent) {
        // Delegate keyup events to the active tool
        get(currentTool)?.onKeyUp?.(e);
    }

	function handleContextMenu(e: MouseEvent) {
		if (e.button === 1) {
			e.preventDefault();
		}
	}

</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
	class="w-full h-screen overflow-hidden relative bg-gray-100 cursor-default" 
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
		style:transform="translate({$viewTransform.posX}px, {$viewTransform.posY}px) scale({$viewTransform.scale})"
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
</div>
