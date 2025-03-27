<script lang="ts">
	import { get } from 'svelte/store';
	import {
        viewTransform,
        screenToCanvasCoordinates,
        createNodeAtPosition,
        nodes,
        layout,
        currentMode, // Need this if checking mode here later
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

		let newScale = currentTransform.scale * (e.deltaY < 0 ? 1.1 : 1 / 1.1);
		newScale = Math.max(0.1, Math.min(newScale, 5));

		const newPosX = mouseX - beforeZoomX * newScale;
		const newPosY = mouseY - beforeZoomY * newScale;

		viewTransform.set({ scale: newScale, posX: newPosX, posY: newPosY });
	}

	function handleMouseDown(e: MouseEvent) {
		if (e.button === 1) { // Middle mouse
			e.preventDefault();
			isPanning = true;
			const currentTransform = get(viewTransform);
			panStartX = e.clientX - currentTransform.posX;
			panStartY = e.clientY - currentTransform.posY;
			if(canvasContainer) canvasContainer.style.cursor = 'grabbing';
		}
        // If left click is on background (not a node), cancel connection
        if (e.button === 0 && e.target === canvas) {
            cancelConnectionProcess();
        }
	}

	function handleMouseMove(e: MouseEvent) {
		if (isPanning) {
			const newPosX = e.clientX - panStartX;
			const newPosY = e.clientY - panStartY;
             viewTransform.set({ scale: $viewTransform.scale, posX: newPosX, posY: newPosY }, { duration: 0 });
		}
		// Node dragging/connecting mousemove handled via window listener in NodeWrapper
	}

	function handleMouseUp(e: MouseEvent) {
		if (isPanning && e.button === 1) {
			isPanning = false;
			if(canvasContainer) canvasContainer.style.cursor = 'grab'; // Reset cursor
		}
        // Node dragging/connecting mouseup handled via window listener in NodeWrapper
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
	on:mousemove={handleMouseMove}
	on:mouseup={handleMouseUp}
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
