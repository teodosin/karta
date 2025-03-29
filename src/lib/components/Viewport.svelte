<script lang="ts">
	import { get } from 'svelte/store';
    import {
        viewTransform,
        screenToCanvasCoordinates,
        createNodeAtPosition,
        nodes,
        layout,
        currentTool, // Need this if checking mode here later
        cancelConnectionProcess // Import cancel function
    } from '$lib/karta/KartaStore';
	import NodeWrapper from './NodeWrapper.svelte';
    import EdgeLayer from './EdgeLayer.svelte';

	let canvasContainer: HTMLElement;
	let canvas: HTMLElement;

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
}

	function handleMouseDown(e: MouseEvent) {
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
			window.addEventListener('mouseup', handleMiddleMouseUp, { once: true });
			return; // Don't delegate middle mouse to tool
		}

		// Delegate left/right clicks on canvas to the active tool
        if (e.target === canvas) { // Ensure click is on canvas background
             get(currentTool).onCanvasMouseDown(e);
             // The tool's onCanvasMouseDown might add window listeners if needed
        }
	}

	// Renamed from handleMouseMove to avoid conflict
	function handleMiddleMouseMove(e: MouseEvent) {
		if (isPanning) {
			const newPosX = e.clientX - panStartX;
			const newPosY = e.clientY - panStartY;
             viewTransform.set({ scale: $viewTransform.scale, posX: newPosX, posY: newPosY }, { duration: 0 });
		}
	}

	// Renamed from handleMouseUp to avoid conflict
	function handleMiddleMouseUp(e: MouseEvent) {
		if (isPanning && e.button === 1) {
			isPanning = false;
			if(canvasContainer) canvasContainer.style.cursor = 'grab'; // Reset cursor
			// Remove window listeners for middle-mouse panning
			window.removeEventListener('mousemove', handleMiddleMouseMove);
			// mouseup listener removed by 'once: true'
		}
	}

	// General mouse move on viewport (not specific to middle mouse pan)
	// This might be useful for tools needing hover effects on the canvas itself
	function handleViewportMouseMove(e: MouseEvent) {
		// Delegate to tool? Maybe not needed yet.
		// get(currentTool).onViewportMouseMove(e);
	}

	// General mouse up on viewport
	function handleViewportMouseUp(e: MouseEvent) {
		// Delegate to tool? Maybe not needed yet.
		// get(currentTool).onViewportMouseUp(e);
	}

	// Handle canvas click (distinct from mousedown)
	function handleCanvasClick(e: MouseEvent) {
		if (e.target === canvas && e.button === 0) { // Only left clicks on background
			get(currentTool).onCanvasClick(e);
		}
	}

    function handleKeyDown(e: KeyboardEvent) {
        if (e.key === 'Tab') {
            e.preventDefault();
            if (!canvasContainer) return;
            // Create at view center for now
            const rect = canvasContainer.getBoundingClientRect();
            const screenX = rect.left + rect.width / 2;
            const screenY = rect.top + rect.height / 2;
            const {x, y} = screenToCanvasCoordinates(screenX, screenY, rect);
            createNodeAtPosition(x, y, 'Text');
        }
         if (e.key === 'Escape') { // Allow cancelling connection with Escape
            cancelConnectionProcess();
        }
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
	class="w-full h-screen overflow-hidden relative bg-gray-100 cursor-grab"
	bind:this={canvasContainer}
	on:mousedown={handleMouseDown}
	on:mousemove={handleViewportMouseMove}
	on:mouseup={handleViewportMouseUp}
	on:click={handleCanvasClick}
	on:wheel={handleWheel}
	on:contextmenu={handleContextMenu}
    tabindex="0"
    on:keydown={handleKeyDown}
>
	<div
		class="w-full h-full relative origin-top-left"
		bind:this={canvas}
		style:transform="translate({$viewTransform.posX}px, {$viewTransform.posY}px) scale({$viewTransform.scale})"
	>
		<!-- Edge Rendering Layer -->
        <EdgeLayer />

		<!-- Node Rendering Layer -->
		{#each [...$nodes.values()] as node (node.id)}
			{@const nodeLayout = $layout.get(node.id)}
			{#if nodeLayout}
				<NodeWrapper nodeId={node.id} initialLayout={nodeLayout} />
			{/if}
		{/each}
	</div>
</div>
