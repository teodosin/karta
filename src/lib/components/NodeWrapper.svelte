<!--
// --- Karta Runtime Component ---
// This file is planned for inclusion in the MIT-licensed `karta_runtime` package.
// It handles node positioning and dynamic component loading.
// It needs to support "Play Mode" interactions based on node attributes.
// Avoid adding editor-specific UI logic here.
-->
<script lang="ts">
	import type { DataNode, ViewNode } from "$lib/types/types";
	import { nodes, updateNodeAttributes } from "$lib/karta/NodeStore";
	import { selectedNodeIds } from "$lib/karta/SelectionStore";
	import { vaultName } from "$lib/karta/VaultStore";
	import { nodeRenameRequestId } from "$lib/karta/UIStateStore";
	import { getNodeComponent } from "$lib/node_types/registry";
	import { tick } from "svelte";
	import { fade } from "svelte/transition";

	export let dataNode: DataNode | undefined;
	export let viewNode: ViewNode;

	let nodeWrapperRef: HTMLElement | null = null;
	let inputElementRef: HTMLInputElement | null = null;

	let isEditingName = false;
	let editedName = "";

	// Dynamically get the component based on ntype - only if dataNode exists
	$: NodeComponent = dataNode ? getNodeComponent(dataNode.ntype) : null;
	
	// Helper function to extract name from path
	function getNameFromPath(path: string): string {
		if (!path) return 'root';
		const segments = path.split('/');
		return segments[segments.length - 1] || '';
	}
	
	// Use viewNode.id for fallback if dataNode is missing (ghost node)
	$: nodeName = dataNode?.attributes?.name?.replace(/^vault/, $vaultName || 'vault') ??
		(dataNode?.path ? getNameFromPath(dataNode.path) : '') ??
		`Deleted Node (${viewNode.id.substring(0, 8)})`;

	$: isSelected = dataNode ? $selectedNodeIds.has(dataNode.id) : false;
	$: isRenamable = dataNode ? !dataNode.attributes?.isSystemNode : false; // Ghost nodes are not renamable
	$: isGhost = !$nodes.has(viewNode.id);

	$: isNameVisible =
		viewNode.attributes?.view_isNameVisible ??
		dataNode?.attributes?.view_isNameVisible ??
		true;
	// Remove hasContext calculation

	async function startEditing() {
		// Can only edit if not a ghost and renamable
		if (isGhost || !isRenamable || !dataNode) return;
		editedName = dataNode.attributes?.name ?? dataNode.ntype ?? "";
		isEditingName = true;
		await tick(); // Wait for input to render
		inputElementRef?.focus();
		inputElementRef?.select();
		// Add listener to handle clicks outside
		window.addEventListener("pointerdown", handleClickOutside, {
			capture: true,
		});
	}

	function handleNameSubmit() {
		// Can only submit if not a ghost, renamable, editing, and dataNode exists
		if (
			!isGhost &&
			isRenamable &&
			isEditingName &&
			dataNode &&
			editedName.trim() &&
			editedName !== (dataNode.attributes?.name ?? dataNode.ntype)
		) {
			updateNodeAttributes(dataNode.id, {
				...dataNode.attributes,
				name: editedName.trim(),
			});
		}
		isEditingName = false;
		// Clean up listener after successful submit
		window.removeEventListener("pointerdown", handleClickOutside, {
			capture: true,
		});
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === "Enter") {
			handleNameSubmit();
		} else if (event.key === "Escape") {
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
		window.removeEventListener("pointerdown", handleClickOutside, {
			capture: true,
		});
	}

	// --- Resize Handle Logic Removed - Handled by SelectionBox ---

	// Reactive statement to listen for rename requests
	$: if ($nodeRenameRequestId === viewNode.id) {
		// Check against viewNode.id
		// Ensure dataNode exists before attempting to rename
		if (dataNode) {
			startEditing(); // Use existing function to start editing
		} else {
		}
		nodeRenameRequestId.set(null); // Reset the request store immediately
	}
</script>

<!-- Render the wrapper if viewNode exists -->
{#if viewNode}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<!-- Use viewNode.id as it's guaranteed for data-id -->
	<div
		bind:this={nodeWrapperRef}
		transition:fade|global
		data-id={viewNode.id}
		class={`
			node-wrapper absolute select-none pointer-events-auto
		`}
		class:selected={isSelected}
		class:ghost-node={isGhost}
		style:width="{viewNode.state.current.width}px"
		style:height="{viewNode.state.current.height}px"
		style:transform="translate({viewNode.state.current.x}px, {viewNode.state
			.current.y}px) scale({viewNode.state.current.scale}) rotate({viewNode
			.state.current.rotation}deg) translateX(-50%) translateY(-50%)"
		on:mouseenter={(e: MouseEvent) => {
			if (!isGhost) nodeWrapperRef?.classList.add("node-hover");
		}}
		on:mouseleave={(e: MouseEvent) =>
			nodeWrapperRef?.classList.remove("node-hover")}
	>
		{#if !isGhost && dataNode}
			<!-- Inner container for the specific node type component - Render only if NOT ghost -->
			<div
				class="node-content relative w-full h-full pointer-events-auto"
			>
				{#if NodeComponent}
					<svelte:component
						this={NodeComponent}
						{dataNode}
						{viewNode}
					/>
				{:else}
					<!-- Fallback rendering if component not found -->
					<div
						class="w-full h-full bg-red-500 flex items-center justify-center text-white font-bold"
					>
						?
					</div>
				{/if}
			</div>

			<!-- External Label & Input - Render only if NOT ghost AND isNameVisible is true -->
			{#if isNameVisible}
				<div
					class="node-label absolute bottom-0 left-1/2 transform -translate-x-1/2 translate-y-full mt-1 px-1.5 py-0.5 bg-gray-900 bg-opacity-80 text-center text-xs rounded pointer-events-auto line-clamp-3"
					style="color: var(--color-text-color);"
					class:cursor-text={isRenamable}
					title={isRenamable
						? "Double-click to rename"
						: "System node (cannot be renamed)"}

					style:width="{Math.min(
						viewNode.state.current.width,
						nodeName.length * 7 + 20
					)}px"
					style:max-width="{viewNode.state.current.width}px"
				>
					{#if isRenamable}
						{#if isEditingName}
							<input
								bind:this={inputElementRef}
								type="text"
								bind:value={editedName}
								on:keydown={handleKeyDown}
								class="bg-gray-900 text-xs p-0 border rounded outline-none focus:ring-1"
								style="color: var(--color-text-color); border-color: var(--color-contrast-color); --tw-ring-color: var(--color-contrast-color); width:{Math.min(
									viewNode.state.current.width - 12,
									Math.max(60, editedName.length * 7 + 10)
								)}px"
								spellcheck="false"
							/>
						{:else}
							<span class="text-xs" on:dblclick={startEditing}>{nodeName}</span>
						{/if}
					{:else}
						<!-- Display name for non-renamable nodes, no interaction -->
						<span class="text-xs">{nodeName}</span>
					{/if}
				</div>
			{/if}
		{:else if isGhost}
			<!-- Render a simple placeholder for ghost nodes -->
			<div
				class="ghost-placeholder w-full h-full flex items-center justify-center text-gray-500 italic text-sm"
			>
				(Deleted)
			</div>
		{/if}
	</div>
{/if}

<style>
	.node-wrapper {
		/* Add touch-action for better mobile/touchpad behavior */
		touch-action: none;
		/* Add transition for potential hover effects */
		transition:
			filter 0.15s ease-in-out,
			opacity 0.15s ease-in-out,
			border 0.15s ease-in-out;
	}

	.node-wrapper.selected {
		/* Add a distinct visual style for selected nodes */
		/* Example: Blue outline */
		outline: 2px solid #3b82f6; /* Tailwind's blue-500 */
		/* Ensure it doesn't conflict with hover or other outlines */
		z-index: 10; /* Bring selected nodes slightly forward if needed */
	}
	/* Apply hover only to non-ghost nodes */
	.node-wrapper:not(.ghost-node).node-hover .node-content {
		/* Example hover effect: slightly brighter */
		/* filter: brightness(1.1); */ /* This filter interferes with backdrop-filter on child nodes like TextNode */
	}
	.node-wrapper:not(.ghost-node).node-hover .node-label {
		/* Make label more prominent on hover */
		background-color: rgba(40, 40, 40, 0.9); /* Darker background */
	}

	.node-label input {
		/* Ensure input fits nicely */
		box-sizing: border-box;
		vertical-align: middle;
	}

	/* Ghost node style */
	.ghost-node {
		opacity: 0.4;
		border: 1px dashed #6b7280; /* Tailwind gray-500 */
		/* pointer-events: none; REMOVED - Allow wrapper interaction */
		background-color: rgba(
			55,
			65,
			81,
			0.2
		); /* Tailwind gray-700 with low opacity */
	}
	/* Style for the placeholder text inside ghost node */
	.ghost-placeholder {
		pointer-events: none; /* Ensure placeholder itself is non-interactive */
	}

	/* Resize Handle Styles Removed - Handled by SelectionBox */
</style>
