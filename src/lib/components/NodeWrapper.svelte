<script lang="ts">
	import type { DataNode, ViewNode } from '$lib/types/types';
	import { currentContextId, updateNodeAttributes } from '$lib/karta/KartaStore'; // Assume updateNodeAttributes exists
	import { getNodeComponent } from '$lib/node_types/registry';
	import { tick } from 'svelte';

	export let dataNode: DataNode;
	export let viewNode: ViewNode;

	let nodeWrapperRef: HTMLElement | null = null; // Reference to the main wrapper div
	let inputElementRef: HTMLInputElement | null = null; // Reference to the input field

	let isEditingName = false;
	let editedName = '';

	// Dynamically get the component based on ntype
	$: NodeComponent = dataNode ? getNodeComponent(dataNode.ntype) : null;
	$: nodeName = dataNode?.attributes?.name ?? dataNode?.ntype ?? 'Unnamed Node'; // Robust fallback

	async function startEditing() {
		editedName = nodeName;
		isEditingName = true;
		await tick(); // Wait for input to render
		inputElementRef?.focus();
		inputElementRef?.select();
	}

	function handleNameSubmit() {
		if (isEditingName && editedName.trim() && editedName !== nodeName) {
			// Call store action to update attributes (includes name uniqueness check implicitly or explicitly)
			updateNodeAttributes(dataNode.id, { ...dataNode.attributes, name: editedName.trim() });
		}
		isEditingName = false;
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			handleNameSubmit();
		} else if (event.key === 'Escape') {
			isEditingName = false; // Cancel editing
		}
	}
</script>

{#if dataNode && viewNode}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		bind:this={nodeWrapperRef}
	       data-id={dataNode.id}
		class={`
			node-wrapper absolute select-none cursor-grab pointer-events-none
			w-[100px] h-[100px]
		`}
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

		<!-- External Label & Input - Positioned below the wrapper -->
		<div
			class="node-label absolute bottom-0 left-1/2 transform -translate-x-1/2 translate-y-full mt-1 px-1.5 py-0.5 bg-gray-700 bg-opacity-80 text-white text-xs rounded whitespace-nowrap cursor-text"
		>
			{#if isEditingName}
				<input
					bind:this={inputElementRef}
					type="text"
					bind:value={editedName}
					on:blur={handleNameSubmit}
					on:keydown={handleKeyDown}
					class="bg-gray-900 text-white text-xs p-0 border border-blue-500 rounded outline-none focus:ring-1 focus:ring-blue-400"
					style:width="{Math.max(60, nodeName.length * 7 + 10)}px"
					spellcheck="false"
				/>
			{:else}
				<span on:dblclick={startEditing} title="Double-click to rename">{nodeName}</span>
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
</style>
