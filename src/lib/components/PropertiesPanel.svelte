<script lang="ts">
	import {
		propertiesPanelVisible,
		propertiesPanelNodeId,
		propertiesPanelPosition,
		propertiesPanelSize,
		propertiesPanelCollapsed,
		setPropertiesPanelVisibility,
		setPropertiesPanelPosition,
		setPropertiesPanelSize,
		togglePropertiesPanelCollapsed,
		nodes, // To get DataNode
		updateNodeAttributes // To update attributes
	} from '$lib/karta/KartaStore';
	import { getNodeTypeDef } from '$lib/node_types/registry'; // To get property schema
	import type { DataNode, PropertyDefinition } from '$lib/types/types';
	import { onDestroy, onMount } from 'svelte';
	import { Move, Minimize2, X } from 'lucide-svelte'; // Icons for header

	// --- Component State ---
	let panelElement: HTMLElement | null = null;
	let headerElement: HTMLElement | null = null;
	let isDragging = false;
	let dragStartX = 0;
	let dragStartY = 0;
	let panelInitialX = 0;
	let panelInitialY = 0;

	// TODO: Add state and logic for resizing

	// --- Reactive Data ---
	$: nodeData = $propertiesPanelNodeId ? $nodes.get($propertiesPanelNodeId) : null;
	$: nodeTypeDef = nodeData ? getNodeTypeDef(nodeData.ntype) : null;
	$: propertySchema = nodeTypeDef?.propertySchema ?? [];
	// Create a set of keys defined in the type-specific schema for quick lookup
	$: typeSpecificKeys = new Set(propertySchema.map(p => p.key));

	// --- Dragging Logic ---
	function handleDragStart(event: PointerEvent) {
		if (!headerElement || !panelElement || !(event.target as HTMLElement)?.closest('.panel-header')) return;
		event.preventDefault(); // Prevent text selection during drag
		isDragging = true;
		panelInitialX = $propertiesPanelPosition.x;
		panelInitialY = $propertiesPanelPosition.y;
		dragStartX = event.clientX;
		dragStartY = event.clientY;
		headerElement.setPointerCapture(event.pointerId);
		headerElement.addEventListener('pointermove', handleDragMove);
		headerElement.addEventListener('pointerup', handleDragEnd);
		headerElement.addEventListener('pointercancel', handleDragEnd); // Handle cancel
		document.body.style.cursor = 'grabbing'; // Indicate dragging
	}

	function handleDragMove(event: PointerEvent) {
		if (!isDragging) return;
		const dx = event.clientX - dragStartX;
		const dy = event.clientY - dragStartY;
		setPropertiesPanelPosition({ x: panelInitialX + dx, y: panelInitialY + dy });
	}

	function handleDragEnd(event: PointerEvent) {
		if (!isDragging || !headerElement) return;
		isDragging = false;
		headerElement.releasePointerCapture(event.pointerId);
		headerElement.removeEventListener('pointermove', handleDragMove);
		headerElement.removeEventListener('pointerup', handleDragEnd);
		headerElement.removeEventListener('pointercancel', handleDragEnd);
		document.body.style.cursor = ''; // Reset cursor
	}

	// --- Attribute Editing ---
	// Simple approach: local state per input, update on blur/enter
	let attributeEdits: Record<string, any> = {};

	function handleAttributeChange(key: string, value: any) {
		if (!nodeData) return;
		// Update local temporary state first if needed, or directly call store action
		console.log(`Updating attribute ${key} to:`, value);
		updateNodeAttributes(nodeData.id, { [key]: value });
		// Consider debouncing or saving on blur/enter instead of every keystroke for text inputs
	}

	// --- Lifecycle ---
	onMount(() => {
		// Potential setup for resizers
	});

	onDestroy(() => {
		// Cleanup drag listeners if component is destroyed while dragging
		if (isDragging && headerElement) {
			// Manually trigger end to release capture and remove listeners
			handleDragEnd(new PointerEvent('pointerup')); // Simulate pointer up
		}
	});

	/*
	Future Considerations:
	- Multiple Panels: This component assumes a single panel controlled by global stores.
	  Refactoring would involve passing panel state (ID, position, size, targetNodeId, locked) as props
	  and managing a list/map of panel states in a dedicated store.
	- Locking: Add a lock button/state to prevent the panel from updating when selection changes.
	- Resizing: Implement resize handles and logic using pointer events.
	- Input Components: Create dedicated components for different property types (string, number, boolean, textarea)
	  to handle validation, specific interactions (e.g., number steppers), etc.
	- Debouncing/Saving Strategy: Refine how attribute changes are saved (e.g., debounce text inputs, save on blur/enter).
	*/
</script>

{#if $propertiesPanelVisible && nodeData}
	<div
		bind:this={panelElement}
		class="properties-panel absolute flex flex-col bg-gray-100 dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg shadow-xl z-40 text-gray-900 dark:text-gray-100"
		style:left="{$propertiesPanelPosition.x}px"
		style:top="{$propertiesPanelPosition.y}px"
		style:width="{$propertiesPanelSize.width}px"
		style:height="{$propertiesPanelCollapsed ? 'auto' : $propertiesPanelSize.height + 'px'}"
		aria-labelledby="properties-panel-title"
	>
		<!-- Header -->
		<div
			bind:this={headerElement}
			class="panel-header flex items-center justify-between p-2 border-b border-gray-300 dark:border-gray-600 bg-gray-200 dark:bg-gray-700 rounded-t-lg cursor-grab select-none"
			on:pointerdown={handleDragStart}
		>
			<span class="flex items-center gap-1 font-semibold text-sm truncate" id="properties-panel-title">
				<Move size={14} class="opacity-50" />
				Properties: {nodeData.attributes.name ?? nodeData.id} ({nodeData.ntype})
			</span>
			<div class="flex items-center gap-1">
				<button
					on:click|stopPropagation={togglePropertiesPanelCollapsed}
					class="p-1 rounded hover:bg-gray-300 dark:hover:bg-gray-600"
					aria-label={$propertiesPanelCollapsed ? 'Expand Panel' : 'Collapse Panel'}
					title={$propertiesPanelCollapsed ? 'Expand Panel' : 'Collapse Panel'}
				>
					<Minimize2 size={14} />
				</button>
				<button
					on:click|stopPropagation={() => setPropertiesPanelVisibility(false)}
					class="p-1 rounded hover:bg-gray-300 dark:hover:bg-gray-600"
					aria-label="Close Panel"
					title="Close Panel"
				>
					<X size={14} />
				</button>
			</div>
		</div>

		<!-- Content (Scrollable) -->
		{#if !$propertiesPanelCollapsed}
			<div class="panel-content flex-grow p-3 overflow-y-auto space-y-4">
				<!-- Attributes Section -->
				<section>
					<h3 class="text-xs font-semibold uppercase text-gray-500 dark:text-gray-400 mb-2">
						Attributes
					</h3>
					<div class="space-y-2">
						{#each Object.entries(nodeData.attributes) as [key, value]}
							<!-- Only show attributes NOT defined in the type-specific schema, and not internal flags -->
							{#if key !== 'isSystemNode' && !typeSpecificKeys.has(key)}
								<div class="flex items-center justify-between gap-2">
									<label for="attr-{key}" class="text-sm capitalize truncate">{key}</label>
									{#if key === 'name'}
										{#if nodeData.attributes.isSystemNode}
											<!-- Read-only display for system node names -->
											<span class="text-sm text-gray-600 dark:text-gray-300 truncate px-2 py-1">
												{value}
											</span>
										{:else}
											<!-- Editable input for non-system node names -->
											<input
												type="text"
												id="attr-{key}"
												value={value}
												on:change={(e) => handleAttributeChange(key, (e.target as HTMLInputElement).value)}
												class="w-full px-2 py-1 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 text-sm"
											/>
										{/if}
									{:else}
										<!-- Basic read-only display for other generic attributes -->
										<span class="text-sm text-gray-600 dark:text-gray-300 truncate px-2 py-1">
											{typeof value === 'object' ? JSON.stringify(value) : value}
										</span>
									{/if}
								</div>
							{/if}
						{/each}
					</div>
				</section>

				<!-- Type Properties Section -->
				{#if propertySchema.length > 0}
					<section>
						<h3 class="text-xs font-semibold uppercase text-gray-500 dark:text-gray-400 mb-2">
							{nodeTypeDef?.displayName ?? nodeData.ntype} Properties
						</h3>
						<div class="space-y-2">
							{#each propertySchema as propDef (propDef.key)}
								<div class="flex flex-col gap-1">
									<label for="prop-{propDef.key}" class="text-sm">{propDef.label}</label>
									{#if propDef.type === 'string'}
										<input
											type="text"
											id="prop-{propDef.key}"
											value={nodeData.attributes[propDef.key] ?? ''}
											on:change={(e) => handleAttributeChange(propDef.key, (e.target as HTMLInputElement).value)}
											class="w-full px-2 py-1 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 text-sm"
										/>
									{:else if propDef.type === 'number'}
										<input
											type="number"
											id="prop-{propDef.key}"
											value={nodeData.attributes[propDef.key] ?? 0}
											on:change={(e) => handleAttributeChange(propDef.key, parseFloat((e.target as HTMLInputElement).value) || 0)}
											class="w-full px-2 py-1 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 text-sm"
										/>
									{:else if propDef.type === 'boolean'}
										<input
											type="checkbox"
											id="prop-{propDef.key}"
											checked={!!nodeData.attributes[propDef.key]}
											on:change={(e) => handleAttributeChange(propDef.key, (e.target as HTMLInputElement).checked)}
											class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"
										/>
									{:else if propDef.type === 'textarea'}
										<textarea
											id="prop-{propDef.key}"
											value={nodeData.attributes[propDef.key] ?? ''}
											on:change={(e) => handleAttributeChange(propDef.key, (e.target as HTMLTextAreaElement).value)}
											class="w-full px-2 py-1 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 text-sm min-h-[60px]"
											rows={3}
										></textarea>
									{/if}
								</div>
							{/each}
						</div>
					</section>
				{/if}

			</div>
		{/if}

		<!-- TODO: Add Resize Handle(s) -->
		<!-- Example: <div class="resize-handle bottom-right absolute bottom-0 right-0 w-3 h-3 cursor-nwse-resize bg-red-500"></div> -->

	</div>
{/if}

<!-- Style block removed, classes applied directly -->