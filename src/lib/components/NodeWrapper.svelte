<script lang="ts">
	import { get } from 'svelte/store';
	import {
        nodes,
        layout,
        viewTransform,
        currentMode,
        updateNodeLayout,
        screenToCanvasCoordinates,
        startConnectionProcess, // Added
        updateTempLinePosition, // Added
        finishConnectionProcess // Added
    } from '$lib/karta/KartaStore';
	import type { ViewNodeLayout, NodeId } from '$lib/karta/KartaStore'; // Import NodeId

	export let nodeId: NodeId;
	export let initialLayout: ViewNodeLayout;

	// Reactive data access
	$: nodeData = $nodes.get(nodeId);
	$: nodeLayout = $layout.get(nodeId) ?? initialLayout;
    $: mode = $currentMode; // Use a reactive variable for the template

	// State for dragging
	let isDraggingNode = false;
	let nodeOffsetX = 0;
	let nodeOffsetY = 0;
    let nodeElementRef: HTMLElement | null = null; // Reference to the div


	function handleNodeMouseDown(e: MouseEvent) {
		if (e.button !== 0) return;
		e.stopPropagation();

		nodeElementRef = e.currentTarget as HTMLElement; // Store reference

		if (mode === 'move') {
			isDraggingNode = true;

			const nodeRect = nodeElementRef.getBoundingClientRect();
			const currentTransform = get(viewTransform);
			nodeOffsetX = (e.clientX - nodeRect.left) / currentTransform.scale;
            nodeOffsetY = (e.clientY - nodeRect.top) / currentTransform.scale;

			nodeElementRef.classList.add('ring-2', 'ring-yellow-400', 'z-10');
			nodeElementRef.style.cursor = 'grabbing';

			window.addEventListener('mousemove', handleDragMouseMove); // Use specific name
			window.addEventListener('mouseup', handleDragMouseUp, { once: true }); // Use specific name

		} else if (mode === 'connect') {
			startConnectionProcess(nodeId);
            // Use different listeners for connect drag
            window.addEventListener('mousemove', handleConnectMouseMove);
            window.addEventListener('mouseup', handleConnectMouseUp, { once: true });
		}
	}

    // --- Dragging Handlers ---
	function handleDragMouseMove(e: MouseEvent) {
		// Check flag just in case listener fires unexpectedly
		if (!isDraggingNode) return;
        e.preventDefault();

        // Need the container rect for coordinate conversion
        // Using document.querySelector is fragile; better ways exist (context API, prop drilling)
        // but this works for now. Assume Viewport has the 'cursor-grab' class.
        const containerEl = document.querySelector('.cursor-grab') as HTMLElement;
        if (!containerEl) return;
        const containerRect = containerEl.getBoundingClientRect();
        const {x: mouseCanvasX, y: mouseCanvasY} = screenToCanvasCoordinates(e.clientX, e.clientY, containerRect);
		const newX = mouseCanvasX - nodeOffsetX;
		const newY = mouseCanvasY - nodeOffsetY;
		updateNodeLayout(nodeId, newX, newY);
	}

	function handleDragMouseUp(e: MouseEvent) {
		// Check flag and button
		if (!isDraggingNode || e.button !== 0) return;
		isDraggingNode = false;

        // Clean up visual feedback if element ref exists
        if(nodeElementRef) {
            nodeElementRef.classList.remove('ring-2', 'ring-yellow-400', 'z-10');
            // Cursor will be reset by reactive `mode` variable in template
        }

		// Remove only the drag listeners
		window.removeEventListener('mousemove', handleDragMouseMove);
        // 'once: true' already removed handleDragMouseUp
	}


    // --- Connecting Handlers ---
    function handleConnectMouseMove(e: MouseEvent) {
        const containerEl = document.querySelector('.cursor-grab') as HTMLElement;
        if (!containerEl) return;
        const containerRect = containerEl.getBoundingClientRect();
        const {x: mouseCanvasX, y: mouseCanvasY} = screenToCanvasCoordinates(e.clientX, e.clientY, containerRect);
        updateTempLinePosition(mouseCanvasX, mouseCanvasY);
    }

    function handleConnectMouseUp(e: MouseEvent) {
        window.removeEventListener('mousemove', handleConnectMouseMove); // Ensure removal

        let targetNodeId: NodeId | null = null;
        let currentElement: HTMLElement | null = e.target as HTMLElement;

        // Traverse up DOM to find a node element with data-id
        while(currentElement) {
            if (currentElement.dataset?.id && currentElement.classList.contains('node')) {
                targetNodeId = currentElement.dataset.id;
                break; // Found it
            }
            // Stop if we hit the canvas container or body
            if (currentElement === document.body || currentElement.classList.contains('cursor-grab')) {
                break;
            }
            currentElement = currentElement.parentElement;
        }

        finishConnectionProcess(targetNodeId);
    }

</script>

{#if nodeData && nodeLayout}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
        bind:this={nodeElementRef}
        data-id={nodeId}
		class="node w-[100px] h-[100px] bg-indigo-600 text-white flex items-center justify-center font-bold rounded absolute select-none shadow-md transition-shadow"
		style:transform="translate({nodeLayout.x}px, {nodeLayout.y}px)"
        style:cursor={mode === 'move' ? (isDraggingNode ? 'grabbing' : 'grab') : 'crosshair'}
		on:mousedown={handleNodeMouseDown}
		on:mouseenter={(e) => nodeElementRef?.classList.add('shadow-lg')}
		on:mouseleave={(e) => nodeElementRef?.classList.remove('shadow-lg')}
	>
		<!-- Basic Node Content -->
		{nodeData.label}
        ({Math.round(nodeLayout.x)}, {Math.round(nodeLayout.y)})
	</div>
{/if}

<style>
    .node {
        /* Add touch-action for better mobile/touchpad behavior */
        touch-action: none;
    }
</style>