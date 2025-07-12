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
	} from "$lib/karta/UIStateStore";
	import {
		nodes,
		updateNodeAttributes,
		updateViewNodeAttribute
	} from "$lib/karta/NodeStore";
	import { contexts, currentContextId } from "$lib/karta/ContextStore"; // Import context stores
	import { getNodeTypeDef } from "$lib/node_types/registry"; // To get property schema
	import { AVAILABLE_FONTS } from "$lib/types/types"; // Import AVAILABLE_FONTS
	import { onDestroy, onMount } from "svelte";
	import ColorPicker from "svelte-awesome-color-picker"; // Import ColorPicker
	import { Move, Minimize2, X } from "lucide-svelte"; // Icons for header
	import ColorPickerPortalWrapper from "./ColorPickerPortalWrapper.svelte"; // Import the portal wrapper

	// --- Component State ---
	let panelElement: HTMLElement | null = null;
	let headerElement: HTMLElement | null = null;
	let isDragging = false;
	let dragStartX = 0;
	let dragStartY = 0;
	let panelInitialX = 0;
	let panelInitialY = 0;
	let headerHeight = 0; // State for measured header height

	// --- Resizing State ---
	let isResizing = false;
	let resizeStartX = 0;
	let resizeStartY = 0;
	let panelInitialWidth = 0;
	let panelInitialHeight = 0;
	let panelInitialResizeX = 0; // Store initial panel X for left-side resize
	let resizeDirection:
		| "left"
		| "right"
		| "bottom"
		| "bottom-left"
		| "bottom-right"
		| null = null;
	const MIN_PANEL_WIDTH = 200; // Minimum width
	const MIN_PANEL_HEIGHT = 100; // Minimum height (allow smaller for just header)
	const HANDLE_SIZE = 6; // px size of invisible handles

	// --- Color Picker State ---
	// Stores the *committed* color value reflecting the node's state
	let fillColorRgb: { r: number; g: number; b: number; a: number } = {
		r: 254,
		g: 249,
		b: 195,
		a: 1,
	};
	let textColorRgb: { r: number; g: number; b: number; a: number } = {
		r: 0,
		g: 0,
		b: 0,
		a: 1,
	};

	// State to control color picker pop-up visibility
	let isFillPickerOpen = false;
	let isTextColorPickerOpen = false;

	// Helper to convert hex or rgba string to RGB object
	function hexOrRgbaToRgb(color: string | undefined): {
		r: number;
		g: number;
		b: number;
		a: number;
	} {
		if (!color) return { r: 0, g: 0, b: 0, a: 1 }; // Default to black if undefined

		// Handle hex color (with or without alpha)
		const hexMatch = color.match(
			/^#([0-9a-fA-F]{3,4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$/,
		);
		if (hexMatch) {
			let hex = hexMatch[1];
			if (hex.length === 3)
				hex = hex[0] + hex[0] + hex[1] + hex[1] + hex[2] + hex[2];
			if (hex.length === 4)
				hex =
					hex[0] +
					hex[0] +
					hex[1] +
					hex[1] +
					hex[2] +
					hex[2] +
					hex[3] +
					hex[3];
			const r = parseInt(hex.substring(0, 2), 16);
			const g = parseInt(hex.substring(2, 4), 16);
			const b = parseInt(hex.substring(4, 6), 16);
			const a =
				hex.length === 8 ? parseInt(hex.substring(6, 8), 16) / 255 : 1;
			return { r, g, b, a };
		}

		// Handle rgba color
		const rgbaMatch = color.match(
			/^rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*(?:,\s*(\d*\.?\d+)\s*)?\)$/,
		);
		if (rgbaMatch) {
			const r = parseInt(rgbaMatch[1], 10);
			const g = parseInt(rgbaMatch[2], 10);
			const b = parseInt(rgbaMatch[3], 10);
			const a = rgbaMatch[4] ? parseFloat(rgbaMatch[4]) : 1;
			return { r, g, b, a };
		}

		return { r: 0, g: 0, b: 0, a: 1 }; // Fallback
	}

	// Helper to convert RGB object to rgba string
	function rgbToRgbaString(rgb: {
		r: number;
		g: number;
		b: number;
		a: number;
	}): string {
		// Clamp values to valid ranges
		const r = Math.max(0, Math.min(255, Math.round(rgb.r)));
		const g = Math.max(0, Math.min(255, Math.round(rgb.g)));
		const b = Math.max(0, Math.min(255, Math.round(rgb.b)));
		const a = Math.max(0, Math.min(1, rgb.a));
		return `rgba(${r}, ${g}, ${b}, ${a})`;
	}

	// Removed handleColorChange function as it's no longer used

	// --- Reactive Data ---
	// Get selected DataNode
	$: selectedDataNode = $propertiesPanelNodeId
		? $nodes.get($propertiesPanelNodeId)
		: null;
	// Derive selected ViewNode from stores
	$: currentCtx = $currentContextId ? $contexts.get($currentContextId) : null;
	$: selectedViewNode =
		$propertiesPanelNodeId && currentCtx
			? currentCtx.viewNodes.get($propertiesPanelNodeId)
			: null;

	$: nodeTypeDef = selectedDataNode ? getNodeTypeDef(selectedDataNode.ntype) : null;
	$: propertySchema = nodeTypeDef?.propertySchema ?? [];

	/**
	 * Creates a reactive `displayAttributes` object.
	 * This is the single source of truth for the UI. It correctly merges attributes
	 * using the cascade: ViewNode (override) -> DataNode (base) -> TypeDef (default).
	 */
	$: displayAttributes = (() => {
		if (!selectedDataNode || !nodeTypeDef) return {};

		const defaults = nodeTypeDef.getDefaultAttributes() ?? {};
		const dataAttrs = selectedDataNode.attributes ?? {};
		const viewAttrs = selectedViewNode?.attributes ?? {};

		// The order is important: start with defaults, layer on data attributes, then finish with view-specific overrides.
		return { ...defaults, ...dataAttrs, ...viewAttrs };
	})();

	// --- Generic Attribute Update Handler ---
	function handleAttributeUpdate(key: string, value: any) {
		if (!selectedDataNode) return;

		// Attributes starting with 'view_' are context-specific and live on the ViewNode.
		if (key.startsWith('view_') || key.startsWith('viewtype_')) {
			if (selectedViewNode) {
				updateViewNodeAttribute(selectedViewNode.id, key, value);
			} else {
				console.warn(`Cannot update view attribute "${key}" - no ViewNode selected.`);
			}
		} else {
			// All other attributes live on the DataNode.
			updateNodeAttributes(selectedDataNode.id, { [key]: value });
		}
	}

	// Removed custom handleClickOutside logic - relying on library behavior with portal

	// --- Dragging Logic ---
	function handleDragStart(event: PointerEvent) {
		// This now fires only on the title span
		const target = event.target as HTMLElement;
		if (!panelElement || !target) return;

		event.preventDefault(); // Prevent text selection during drag
		isDragging = true;
		panelInitialX = $propertiesPanelPosition.x;
		panelInitialY = $propertiesPanelPosition.y;
		dragStartX = event.clientX;
		dragStartY = event.clientY;

		target.setPointerCapture(event.pointerId); // Capture on the span itself
		document.addEventListener("pointermove", handleDragMove); // Listen on document
		document.addEventListener("pointerup", handleDragEnd, { once: true });
		document.addEventListener("pointercancel", handleDragEnd, {
			once: true,
		});
		document.body.style.cursor = "grabbing"; // Indicate dragging
	}

	function handleDragMove(event: PointerEvent) {
		if (!isDragging || !panelElement) return; // Added check for panelElement

		const dx = event.clientX - dragStartX;
		const dy = event.clientY - dragStartY;

		let newX = panelInitialX + dx;
		let newY = panelInitialY + dy;

		// Get panel dimensions (use clientWidth/Height for actual rendered size)
		const panelWidth = panelElement.clientWidth;
		const panelHeight = panelElement.clientHeight;

		// Clamp position within window bounds
		const maxX = window.innerWidth - panelWidth;
		const maxY = window.innerHeight - panelHeight;

		newX = Math.max(0, Math.min(newX, maxX));
		newY = Math.max(0, Math.min(newY, maxY));

		setPropertiesPanelPosition({ x: newX, y: newY });
	}

	function handleDragEnd(event: PointerEvent) {
		if (!isDragging) return;
		isDragging = false;
		// Release capture from the element that initiated the drag (the span)
		try {
			(event.target as HTMLElement)?.releasePointerCapture?.(
				event.pointerId,
			);
		} catch (e) {
			/* ignore */
		}

		document.removeEventListener("pointermove", handleDragMove);
		// pointerup/cancel listeners were added to document with { once: true }
		document.body.style.cursor = ""; // Reset cursor
	}

	// --- Attribute Editing ---
	// Simple approach: local state per input, update on blur/enter
	let attributeEdits: Record<string, any> = {};

	function handleAttributeChange(key: string, value: any) {
		if (!selectedDataNode) return;
		// Update local temporary state first if needed, or directly call store action
		// Use the specific updateNodeAttributes for DataNode attributes
		updateNodeAttributes(selectedDataNode.id, { [key]: value });
		// Consider debouncing or saving on blur/enter instead of every keystroke for text inputs
	}

	// --- Lifecycle ---
	onMount(() => {
		// Ensure initial size is set if needed, though store should handle it.
		// Could add resize observers here if needed later.
	});

	onDestroy(() => {
		// Cleanup drag listeners if component is destroyed while dragging
		if (isDragging && headerElement) {
			// Manually trigger end to release capture and remove listeners
			handleDragEnd(new PointerEvent("pointerup")); // Simulate pointer up for drag
		}
		// Cleanup resize listeners if component is destroyed while resizing
		if (isResizing && panelElement) {
			handleResizeEnd(new PointerEvent("pointerup")); // Simulate pointer up for resize
		}
	});

	// Action to measure header height
	function measureHeaderHeight(node: HTMLElement) {
		// Run after mount and updates
		const updateHeight = () => {
			if (node) {
				headerHeight = node.offsetHeight;
			}
		};

		// Initial measurement after mount
		requestAnimationFrame(updateHeight);

		// Return object with update method if needed for resize observer later
		return {
			// update: updateHeight // Could be used with ResizeObserver if header size changes dynamically
		};
	}

	// --- Resizing Logic ---
	function handleResizeStart(event: PointerEvent) {
		const target = event.target as HTMLElement;
		const direction = target.dataset.direction as typeof resizeDirection;
		if (!panelElement || !direction) return;

		event.preventDefault();
		event.stopPropagation(); // Prevent drag start on header
		isResizing = true;
		resizeDirection = direction;
		panelInitialWidth = panelElement.offsetWidth; // Use offsetWidth for actual rendered size
		panelInitialHeight = panelElement.offsetHeight;
		panelInitialResizeX = $propertiesPanelPosition.x; // Store initial X for left resize
		resizeStartX = event.clientX;
		resizeStartY = event.clientY;

		const handle = event.target as HTMLElement;
		target.setPointerCapture(event.pointerId);
		// Add listeners to the document to capture events outside the handle
		document.addEventListener("pointermove", handleResizeMove);
		document.addEventListener("pointerup", handleResizeEnd, { once: true }); // Use once for cleanup
		document.addEventListener("pointercancel", handleResizeEnd, {
			once: true,
		});

		// Set cursor based on direction
		switch (direction) {
			case "left":
			case "right":
				document.body.style.cursor = "ew-resize";
				break;
			case "bottom":
				document.body.style.cursor = "ns-resize";
				break;
			case "bottom-left":
				document.body.style.cursor = "nesw-resize"; // Note: opposite corner for diagonal
				break;
			case "bottom-right":
				document.body.style.cursor = "nwse-resize";
				break;
		}
	}

	function handleResizeMove(event: PointerEvent) {
		if (!isResizing || !resizeDirection) return;

		const dx = event.clientX - resizeStartX;
		const dy = event.clientY - resizeStartY;

		let newX = $propertiesPanelPosition.x;
		let newY = $propertiesPanelPosition.y;
		let newWidth = panelInitialWidth;
		let newHeight = panelInitialHeight;

		// Calculate new dimensions and position based on direction
		if (resizeDirection.includes("left")) {
			newWidth = panelInitialWidth - dx;
			if (newWidth >= MIN_PANEL_WIDTH) {
				newX = panelInitialResizeX + dx;
			} else {
				newWidth = MIN_PANEL_WIDTH; // Prevent shrinking beyond min and moving left edge
				newX =
					panelInitialResizeX + (panelInitialWidth - MIN_PANEL_WIDTH);
			}
		} else if (resizeDirection.includes("right")) {
			newWidth = panelInitialWidth + dx;
		}

		if (resizeDirection.includes("bottom")) {
			newHeight = panelInitialHeight + dy;
		}

		// Apply minimum size constraints again after potential adjustments
		newWidth = Math.max(MIN_PANEL_WIDTH, newWidth);
		newHeight = Math.max(MIN_PANEL_HEIGHT, newHeight);

		// Apply boundary constraints (prevent resizing outside window)
		const maxX = window.innerWidth;
		const maxY = window.innerHeight;

		if (newX < 0) {
			newWidth += newX; // Shrink width if moving left edge past boundary
			newX = 0;
		}
		if (newY < 0) {
			// Should not happen with current handles, but good practice
			newHeight += newY;
			newY = 0;
		}
		if (newX + newWidth > maxX) {
			newWidth = maxX - newX;
		}
		if (newY + newHeight > maxY) {
			newHeight = maxY - newY;
		}

		// Final check on minimums after boundary adjustments
		newWidth = Math.max(MIN_PANEL_WIDTH, newWidth);
		newHeight = Math.max(MIN_PANEL_HEIGHT, newHeight);

		// Update store
		setPropertiesPanelPosition({ x: newX, y: newY }); // Update position if left edge moved
		setPropertiesPanelSize({ width: newWidth, height: newHeight });
	}

	function handleResizeEnd(event: PointerEvent) {
		if (!isResizing) return;

		// Release pointer capture if it exists on the document or target
		try {
			if (document.pointerLockElement) {
				document.exitPointerLock();
			}
			// Attempt to release capture specifically if possible, might error if element removed
			(event.target as HTMLElement)?.releasePointerCapture?.(
				event.pointerId,
			);
		} catch (e) {
			// Ignore errors, capture might already be released or element gone
		}

		isResizing = false;
		resizeDirection = null;
		document.removeEventListener("pointermove", handleResizeMove);
		// 'pointerup' and 'pointercancel' listeners were added to document with { once: true }
		document.body.style.cursor = ""; // Reset cursor
		// Check if the pointer capture target exists before trying to release
		// This handles cases where the element might be removed during the operation
		if (document.pointerLockElement) {
			try {
				document.exitPointerLock(); // General cleanup
			} catch (e) {
				/* ignore */
			}
		}
		// Attempt to release capture specifically if possible, might error if element removed
		try {
			(event.target as HTMLElement)?.releasePointerCapture?.(
				event.pointerId,
			);
		} catch (e) {
			/* ignore */
		}

		isResizing = false;
		resizeDirection = null;
		document.removeEventListener("pointermove", handleResizeMove);
		document.body.style.cursor = ""; // Reset cursor
	}

	/*
	Future Considerations:
	- Multiple Panels: Refactor to use props and manage state centrally.
	- Locking: Add lock state.
	- Input Components: Create dedicated input components.
	- Debouncing/Saving Strategy: Refine attribute saving.
	*/
</script>

<div class="w-full h-full">
	{#if $propertiesPanelVisible && selectedDataNode}
		<div
			bind:this={panelElement}
			class="properties-panel absolute flex flex-col bg-panel-bg dark:bg-panel-bg border border-panel-bg/50 rounded-lg shadow-xl z-40 text-gray-900 dark:text-gray-100 overflow-hidden"
			style:height={$propertiesPanelCollapsed
				? headerHeight > 0
					? `${headerHeight}px`
					: "40px"
				: `${$propertiesPanelSize.height}px`}
			style:left="{$propertiesPanelPosition.x}px"
			style:top="{$propertiesPanelPosition.y}px"
			style:width="{$propertiesPanelSize.width}px"
			aria-labelledby="properties-panel-title"
		>
			<!-- Header -->
			<div
				bind:this={headerElement}
				class="panel-header flex items-center justify-between p-2 border-b border-gray-300 dark:border-gray-600 bg-panel-bg dark:bg-panel-bg rounded-t-lg select-none"
				use:measureHeaderHeight
			>
				<span
					class="flex-grow flex items-center gap-1 font-semibold text-sm truncate cursor-grab pr-2"
					id="properties-panel-title"
					on:pointerdown={handleDragStart}
				>
					{selectedDataNode.attributes.name ?? selectedDataNode.id} ({selectedDataNode.ntype})
				</span>
				<div class="flex items-center gap-1">
					<button
						on:click={() => {
							togglePropertiesPanelCollapsed();
						}}
						class="p-1 rounded hover:bg-gray-300 dark:hover:bg-gray-600"
						aria-label={$propertiesPanelCollapsed
							? "Expand Panel"
							: "Collapse Panel"}
						title={$propertiesPanelCollapsed
							? "Expand Panel"
							: "Collapse Panel"}
					>
						<Minimize2 size={14} />
					</button>
				</div>
			</div>

			<!-- Content (Scrollable) -->
			{#if !$propertiesPanelCollapsed}
				<!-- Use #if block to conditionally render content -->
				<div
					class="panel-content flex-grow p-3 overflow-y-auto space-y-4 [&::-webkit-scrollbar]:w-2 [&::-webkit-scrollbar-track]:bg-transparent [&::-webkit-scrollbar-thumb]:bg-gray-600 [&::-webkit-scrollbar-thumb]:rounded-full"
				>
					<!-- Generic Properties Section (Schema-Driven) -->
					<section>
						<h3
							class="text-xs font-semibold uppercase text-gray-500 dark:text-gray-400 mb-2"
						>
							Properties
						</h3>
						<div class="space-y-3">
							<!-- Name is always displayed first and is special -->
							<div class="flex items-center justify-between gap-2">
								<label for="prop-name" class="text-sm capitalize truncate">Name</label>
								{#if displayAttributes.isSystemNode}
									<span class="text-sm text-gray-500 dark:text-gray-400 truncate px-2 py-1">
										{displayAttributes.name ?? ''}
									</span>
								{:else}
									<input
										type="text"
										id="prop-name"
										value={displayAttributes.name ?? ''}
										on:change={(e) => handleAttributeUpdate('name', (e.target as HTMLInputElement).value)}
										class="w-full px-2 py-1 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 text-sm"
									/>
								{/if}
							</div>

							<!-- Render other properties from the schema -->
							{#each propertySchema as prop (prop.key)}
								<div class="flex items-center justify-between gap-2">
									<label for="prop-{prop.key}" class="text-sm truncate">{prop.label}</label>

									<!-- Boolean Type -->
									{#if prop.type === 'boolean'}
										<input
											type="checkbox"
											id="prop-{prop.key}"
											checked={displayAttributes[prop.key] ?? false}
											on:change={(e) => handleAttributeUpdate(prop.key, (e.target as HTMLInputElement).checked)}
											class="form-checkbox h-4 w-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
										/>
									{/if}

									<!-- Number Type -->
									{#if prop.type === 'number'}
										<input
											type="number"
											id="prop-{prop.key}"
											value={displayAttributes[prop.key] ?? 0}
											on:change={(e) => handleAttributeUpdate(prop.key, parseFloat((e.target as HTMLInputElement).value) || 0)}
											class="w-20 px-2 py-1 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 text-sm"
										/>
									{/if}

									<!-- Font Type -->
									{#if prop.type === 'font'}
										<select
											id="prop-{prop.key}"
											value={displayAttributes[prop.key] ?? 'Nunito'}
											on:change={(e) => handleAttributeUpdate(prop.key, (e.target as HTMLSelectElement).value)}
											class="w-full px-2 py-1 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 text-sm"
										>
											{#each AVAILABLE_FONTS as font}
												<option value={font}>{font}</option>
											{/each}
										</select>
									{/if}

									<!-- Color Type -->
									{#if prop.type === 'color'}
										<ColorPicker
											rgb={hexOrRgbaToRgb(displayAttributes[prop.key])}
											onInput={(data) => {
												if (data.rgb) {
													handleAttributeUpdate(prop.key, rgbToRgbaString(data.rgb));
												}
											}}
											position="responsive"
											components={{ wrapper: ColorPickerPortalWrapper }}
											disableCloseClickOutside={true}
										/>
									{/if}
								</div>
							{/each}
						</div>
					</section>
				</div>
				<!-- End of panel-content div -->
			{/if}
			<!-- End of #if !$propertiesPanelCollapsed -->
			<!-- Invisible Resize Handles -->
			{#if !$propertiesPanelCollapsed}
				<!-- Left -->
				<div
					data-direction="left"
					class="absolute top-0 left-0 h-full cursor-ew-resize"
					style="width: {HANDLE_SIZE}px;"
					on:pointerdown|stopPropagation={handleResizeStart}
				></div>
				<!-- Right -->
				<div
					data-direction="right"
					class="absolute top-0 right-0 h-full cursor-ew-resize"
					style="width: {HANDLE_SIZE}px;"
					on:pointerdown|stopPropagation={handleResizeStart}
				></div>
				<!-- Bottom -->
				<div
					data-direction="bottom"
					class="absolute bottom-0 left-0 w-full cursor-ns-resize"
					style="height: {HANDLE_SIZE}px;"
					on:pointerdown|stopPropagation={handleResizeStart}
				></div>
				<!-- Bottom-Left Corner -->
				<div
					data-direction="bottom-left"
					class="absolute bottom-0 left-0 cursor-nesw-resize"
					style="width: {HANDLE_SIZE * 2}px; height: {HANDLE_SIZE *
						2}px;"
					on:pointerdown|stopPropagation={handleResizeStart}
				></div>
				<!-- Bottom-Right Corner -->
				<div
					data-direction="bottom-right"
					class="absolute bottom-0 right-0 cursor-nwse-resize"
					style="width: {HANDLE_SIZE * 2}px; height: {HANDLE_SIZE *
						2}px;"
					on:pointerdown|stopPropagation={handleResizeStart}
				></div>
			{/if}
		</div>
	{/if}
	<div
		id="portal"
		class="absolute top-20 left-0 z-50"
	></div>
</div>

<!-- Style block removed, classes applied directly -->
<style>
	:global(.dark) {
		--cp-bg-color: #333;
		--cp-border-color: white;
		--cp-text-color: white;
		--cp-input-color: #555;
		--cp-button-hover-color: #777;
	}
</style>
