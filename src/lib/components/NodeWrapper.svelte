<!--
// --- Karta Runtime Component ---
// This file is planned for inclusion in the MIT-licensed `karta_runtime` package.
// It handles node positioning and dynamic component loading.
// It needs to support "Play Mode" interactions based on node attributes.
// Avoid adding editor-specific UI logic here.
-->
<script lang="ts">
	import type { DataNode, ViewNode } from '$lib/types/types';
	import { currentContextId, currentViewNodes } from '$lib/karta/ContextStore'; // Import context-related stores
	import { updateNodeAttributes } from '$lib/karta/NodeStore'; // Import node attribute action
	import { selectedNodeIds } from '$lib/karta/SelectionStore'; // Import selection store
	import { nodeRenameRequestId } from '$lib/karta/UIStateStore'; // Import rename request store
	import { getNodeComponent } from '$lib/node_types/registry';
	import { tick } from 'svelte';
	import { fade } from 'svelte/transition'; // Import fade transition
	// Removed startResize import as handles are now in SelectionBox

	export let dataNode: DataNode;
	export let viewNode: ViewNode;

	let nodeWrapperRef: HTMLElement | null = null; // Reference to the main wrapper div
	let inputElementRef: HTMLInputElement | null = null; // Reference to the input field

	let isEditingName = false;
	let editedName = '';

	// Dynamically get the component based on ntype
	$: NodeComponent = dataNode ? getNodeComponent(dataNode.ntype) : null;
	$: nodeName = dataNode?.attributes?.name ?? dataNode?.ntype ?? 'Unnamed Node'; // Robust fallback
	$: isSelected = dataNode ? $selectedNodeIds.has(dataNode.id) : false; // Check if this node is selected
	$: isRenamable = !dataNode?.attributes?.isSystemNode; // Check the attribute

	async function startEditing() {
		if (!isRenamable) return; // Prevent editing if not renamable
		editedName = nodeName;
		isEditingName = true;
		await tick(); // Wait for input to render
		inputElementRef?.focus();
		inputElementRef?.select();
		// Add listener to handle clicks outside
		window.addEventListener('pointerdown', handleClickOutside, { capture: true });
	}

	function handleNameSubmit() {
		// Only submit if renamable and editing
		if (isRenamable && isEditingName && editedName.trim() && editedName !== nodeName) {
			// Store action now handles uniqueness and system node checks
			updateNodeAttributes(dataNode.id, { ...dataNode.attributes, name: editedName.trim() });
		}
		isEditingName = false;
		// Clean up listener after successful submit
		window.removeEventListener('pointerdown', handleClickOutside, { capture: true });
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			handleNameSubmit();
		} else if (event.key === 'Escape') {
			cancelEditing(); // Use cancel function for Escape
		}
	}

	function handleClickOutside(event: PointerEvent) {
		if (inputElementRef && event.target !== inputElementRef) {
			cancelEditing();
		}
	}

	function cancelEditing() {
		if (!isEditingName) return; // Avoid removing listener multiple times
		isEditingName = false;
		window.removeEventListener('pointerdown', handleClickOutside, { capture: true });
	}

	// --- Resize Handle Logic Removed - Handled by SelectionBox ---

	// Reactive statement to listen for rename requests
	$: if ($nodeRenameRequestId === dataNode?.id) {
		console.log(`[NodeWrapper ${dataNode.id}] Received rename request via store.`);
		startEditing(); // Use existing function to start editing
		nodeRenameRequestId.set(null); // Reset the request store immediately
	}
</script>

{#if dataNode && viewNode}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		bind:this={nodeWrapperRef}
		transition:fade|global
	    data-id={dataNode.id}
		class={`
			node-wrapper absolute select-none cursor-grab pointer-events-none
		`}
		class:selected={isSelected}
		style:width="{viewNode.state.current.width}px"
		style:height="{viewNode.state.current.height}px"
		style:transform="translate({viewNode.state.current.x}px, {viewNode.state.current.y}px) scale({viewNode.state.current.scale}) rotate({viewNode.state.current.rotation}deg) translateX(-50%) translateY(-50%)"
		on:mouseenter={(e: MouseEvent) => nodeWrapperRef?.classList.add('node-hover')}
		on:mouseleave={(e: MouseEvent) => nodeWrapperRef?.classList.remove('node-hover')}
	>
		<!-- Inner container for the specific node type component -->
		<div class="node-content relative w-full h-full pointer-events-auto">
			{#if NodeComponent}
				<svelte:component this={NodeComponent} {dataNode} {viewNode} />
			{:else}
				<!-- Fallback rendering if component not found -->
				<div class="w-full h-full bg-red-500 flex items-center justify-center text-white font-bold">?</div>
			{/if}
		</div>

		<!-- Resize Handles Removed - Handled by SelectionBox -->

		<!-- External Label & Input - Positioned below the wrapper -->
		<div
			class="node-label absolute bottom-0 left-1/2 transform -translate-x-1/2 translate-y-full mt-1 px-1.5 py-0.5 bg-gray-700 bg-opacity-80 text-white text-xs rounded whitespace-nowrap pointer-events-auto"
			class:cursor-text={isRenamable}
			title={isRenamable ? 'Double-click to rename' : 'System node (cannot be renamed)'}
		>
			{#if isRenamable}
				{#if isEditingName}
					<input
						bind:this={inputElementRef}
						type="text"
						bind:value={editedName}
						on:keydown={handleKeyDown}
						class="bg-gray-900 text-white text-xs p-0 border border-blue-500 rounded outline-none focus:ring-1 focus:ring-blue-400"
						style:width="{Math.max(60, nodeName.length * 7 + 10)}px"
						spellcheck="false"
					/>
				{:else}
					<span on:dblclick={startEditing}>{nodeName}</span>
				{/if}
			{:else}
				<!-- Display name for non-renamable nodes, no interaction -->
				<span>{nodeName}</span>
			{/if}
		</div>
	</div>
{/if}

<style>
	.node-wrapper {
        /* Add touch-action for better mobile/touchpad behavior */
        touch-action: none;
		/* Add transition for potential hover effects */
		transition: filter 0.15s ease-in-out;
	}

	.node-wrapper.selected {
		/* Add a distinct visual style for selected nodes */
		/* Example: Blue outline */
		outline: 2px solid #3b82f6; /* Tailwind's blue-500 */
		/* Ensure it doesn't conflict with hover or other outlines */
		z-index: 10; /* Bring selected nodes slightly forward if needed */
	}
	.node-wrapper.node-hover .node-content {
		/* Example hover effect: slightly brighter */
 		filter: brightness(1.1);
	}
	.node-wrapper.node-hover .node-label {
		/* Make label more prominent on hover */
		background-color: rgba(40, 40, 40, 0.9); /* Darker background */
	}

	.node-label input {
		/* Ensure input fits nicely */
		box-sizing: border-box;
		height: 1.25rem; /* Match typical text line height */
		vertical-align: middle;
	}

	/* Resize Handle Styles Removed - Handled by SelectionBox */
</style>
