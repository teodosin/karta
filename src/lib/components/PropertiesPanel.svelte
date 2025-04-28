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
		togglePropertiesPanelCollapsed
	} from '$lib/karta/UIStateStore';
	import { nodes, updateNodeAttributes, updateViewNodeAttribute } from '$lib/karta/NodeStore'; // Import new action
	import { contexts, currentContextId } from '$lib/karta/ContextStore'; // Import context stores
	import { getNodeTypeDef } from '$lib/node_types/registry'; // To get property schema
	import type { DataNode, PropertyDefinition, ViewNode, AvailableFont } from '$lib/types/types'; // Import ViewNode, AvailableFont
	import { AVAILABLE_FONTS } from '$lib/types/types'; // Import AVAILABLE_FONTS
	import { onDestroy, onMount } from 'svelte';
	import ColorPicker from 'svelte-awesome-color-picker'; // Import ColorPicker
	import { Move, Minimize2, X } from 'lucide-svelte'; // Icons for header

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
	let resizeDirection: 'left' | 'right' | 'bottom' | 'bottom-left' | 'bottom-right' | null = null;
	const MIN_PANEL_WIDTH = 200; // Minimum width
	const MIN_PANEL_HEIGHT = 100; // Minimum height (allow smaller for just header)
	const HANDLE_SIZE = 6; // px size of invisible handles

	// --- Color Picker State ---
	let fillColorRgb: { r: number; g: number; b: number; a: number } = { r: 254, g: 249, b: 195, a: 1 }; // Derived from store/node
	let textColorRgb: { r: number; g: number; b: number; a: number } = { r: 0, g: 0, b: 0, a: 1 }; // Derived from store/node

	// Intermediate state for color pickers to avoid store updates during drag
	let intermediateFillColorRgb = { ...fillColorRgb };
	let intermediateTextColorRgb = { ...textColorRgb };

	// State to control color picker pop-up visibility
	let isFillPickerOpen = false;
	let isTextColorPickerOpen = false;

	// Helper to convert hex or rgba string to RGB object
	function hexOrRgbaToRgb(color: string | undefined): { r: number; g: number; b: number; a: number } {
		if (!color) return { r: 0, g: 0, b: 0, a: 1 }; // Default to black if undefined

		// Handle hex color (with or without alpha)
		const hexMatch = color.match(/^#([0-9a-fA-F]{3,4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$/);
		if (hexMatch) {
			let hex = hexMatch[1];
			if (hex.length === 3) hex = hex[0] + hex[0] + hex[1] + hex[1] + hex[2] + hex[2];
			if (hex.length === 4) hex = hex[0] + hex[0] + hex[1] + hex[1] + hex[2] + hex[2] + hex[3] + hex[3];
			const r = parseInt(hex.substring(0, 2), 16);
			const g = parseInt(hex.substring(2, 4), 16);
			const b = parseInt(hex.substring(4, 6), 16);
			const a = hex.length === 8 ? parseInt(hex.substring(6, 8), 16) / 255 : 1;
			return { r, g, b, a };
		}

		// Handle rgba color
		const rgbaMatch = color.match(/^rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*(?:,\s*(\d*\.?\d+)\s*)?\)$/);
		if (rgbaMatch) {
			const r = parseInt(rgbaMatch[1], 10);
			const g = parseInt(rgbaMatch[2], 10);
			const b = parseInt(rgbaMatch[3], 10);
			const a = rgbaMatch[4] ? parseFloat(rgbaMatch[4]) : 1;
			return { r, g, b, a };
		}

		console.warn("Could not parse color string:", color);
		return { r: 0, g: 0, b: 0, a: 1 }; // Fallback
	}

	// Helper to convert RGB object to rgba string
	function rgbToRgbaString(rgb: { r: number; g: number; b: number; a: number }): string {
		// Clamp values to valid ranges
		const r = Math.max(0, Math.min(255, Math.round(rgb.r)));
		const g = Math.max(0, Math.min(255, Math.round(rgb.g)));
		const b = Math.max(0, Math.min(255, Math.round(rgb.b)));
		const a = Math.max(0, Math.min(1, rgb.a));
		return `rgba(${r}, ${g}, ${b}, ${a})`;
	}

	// Single handler for color changes
	// Adjusted handler for onInput event
	function handleColorChange(key: 'karta_fillColor' | 'karta_textColor', eventDetail: { rgb: { r: number; g: number; b: number; a: number } | null }) {
		if (selectedViewNode && eventDetail.rgb) {
			const rgbaString = rgbToRgbaString(eventDetail.rgb);
			updateViewNodeAttribute(selectedViewNode.id, key, rgbaString);
		}
	}

	// --- Reactive Data ---
	// Get selected DataNode
	$: selectedDataNode = $propertiesPanelNodeId ? $nodes.get($propertiesPanelNodeId) : null;
	// Derive selected ViewNode from stores
	$: currentCtx = $currentContextId ? $contexts.get($currentContextId) : null;
	$: selectedViewNode = $propertiesPanelNodeId && currentCtx ? currentCtx.viewNodes.get($propertiesPanelNodeId) : null;


	$: nodeTypeDef = selectedDataNode ? getNodeTypeDef(selectedDataNode.ntype) : null;
	$: propertySchema = nodeTypeDef?.propertySchema ?? [];
	// Create a set of keys defined in the type-specific schema for quick lookup
	$: typeSpecificKeys = new Set(propertySchema.map(p => p.key));

	// --- Text Node Style State ---
	let currentFillColor: string = '#FEF9C3'; // Default fallback
	let currentTextColor: string = '#000000'; // Default fallback
	let currentFont: AvailableFont = 'Nunito'; // Default fallback (calculated effective value)
	let currentFontSize = 16; // Default fallback for font size (calculated effective value)
	let selectedFontValue: AvailableFont = 'Nunito'; // Local state for select binding
	let selectedFontSizeValue: number = 16; // Local state for input binding
	const FALLBACK_FONT_SIZE = 16; // Define fallback globally in script

	$: {
		if (selectedViewNode && selectedDataNode && selectedDataNode.ntype === 'text') {
			const viewAttrs = selectedViewNode.attributes;
			const dataAttrs = selectedDataNode.attributes;
			const FALLBACK_FILL_COLOR = '#FEF9C3';
			const FALLBACK_TEXT_COLOR = '#000000';
			const FALLBACK_FONT: AvailableFont = 'Nunito';
			// Removed declaration from here

			// Get effective colors and font
			const effectiveFillColor = viewAttrs?.karta_fillColor ?? dataAttrs?.karta_fillColor ?? FALLBACK_FILL_COLOR;
			const effectiveTextColor = viewAttrs?.karta_textColor ?? dataAttrs?.karta_textColor ?? FALLBACK_TEXT_COLOR;
			currentFont = viewAttrs?.karta_font ?? dataAttrs?.karta_font ?? FALLBACK_FONT;
			currentFontSize = viewAttrs?.karta_fontSize ?? dataAttrs?.karta_fontSize ?? FALLBACK_FONT_SIZE; // Calculate effective font size

			// Update local state for bindings based on calculated effective values
			selectedFontValue = currentFont;
			selectedFontSizeValue = currentFontSize;

			// Initialize color picker state AND intermediate state from effective colors
			fillColorRgb = hexOrRgbaToRgb(effectiveFillColor);
			textColorRgb = hexOrRgbaToRgb(effectiveTextColor);
			intermediateFillColorRgb = { ...fillColorRgb }; // Initialize intermediate state
			intermediateTextColorRgb = { ...textColorRgb }; // Initialize intermediate state
		} else {
			// Reset when node changes or is not text
			fillColorRgb = { r: 254, g: 249, b: 195, a: 1 };
			textColorRgb = { r: 0, g: 0, b: 0, a: 1 };
			intermediateFillColorRgb = { ...fillColorRgb };
			intermediateTextColorRgb = { ...textColorRgb };
			currentFont = 'Nunito';
			currentFontSize = 16; // Reset font size state
			selectedFontValue = 'Nunito'; // Reset binding state
			selectedFontSizeValue = 16; // Reset binding state
		}
	}

	// --- Apply Intermediate Color on Picker Close ---
	let prevIsFillPickerOpen = isFillPickerOpen;
	$: {
		if (prevIsFillPickerOpen && !isFillPickerOpen && selectedViewNode) {
			// Picker was just closed, apply the intermediate color if changed
			const originalColorString = rgbToRgbaString(fillColorRgb);
			const finalColorString = rgbToRgbaString(intermediateFillColorRgb);
			if (originalColorString !== finalColorString) {
				console.log(`Applying fill color change: ${finalColorString}`);
				updateViewNodeAttribute(selectedViewNode.id, 'karta_fillColor', finalColorString);
				fillColorRgb = { ...intermediateFillColorRgb }; // Update original state to match
			}
		}
		prevIsFillPickerOpen = isFillPickerOpen; // Update previous state for next check
	}

	let prevIsTextColorPickerOpen = isTextColorPickerOpen;
	$: {
		if (prevIsTextColorPickerOpen && !isTextColorPickerOpen && selectedViewNode) {
			// Picker was just closed, apply the intermediate color if changed
			const originalColorString = rgbToRgbaString(textColorRgb);
			const finalColorString = rgbToRgbaString(intermediateTextColorRgb);
			if (originalColorString !== finalColorString) {
				console.log(`Applying text color change: ${finalColorString}`);
				updateViewNodeAttribute(selectedViewNode.id, 'karta_textColor', finalColorString);
				textColorRgb = { ...intermediateTextColorRgb }; // Update original state to match
			}
		}
		prevIsTextColorPickerOpen = isTextColorPickerOpen; // Update previous state for next check
	}


	// --- Text Node Style Handlers ---
		function handleFontChange(event: Event) {
			if (selectedViewNode) {
			const target = event.target as HTMLSelectElement;
			// Ensure the value is a valid AvailableFont before updating
			const selectedValue = target.value as AvailableFont;
			if (AVAILABLE_FONTS.includes(selectedValue)) {
				updateViewNodeAttribute(selectedViewNode.id, 'karta_font', selectedValue);
			} else {
				console.warn("Invalid font selected:", target.value);
			}
		}
	}

	// --- Click Outside Logic for Color Pickers ---
	function handleClickOutside(event: MouseEvent) {
		// Check if the click is outside the panel element itself
		if (panelElement && !panelElement.contains(event.target as Node)) {
			// If clicked outside the panel, close any open color pickers
			if (isFillPickerOpen || isTextColorPickerOpen) {
				isFillPickerOpen = false;
				isTextColorPickerOpen = false;
			}
		}
	}

	// Add/remove click outside listener based on picker state
	$: {
		if (isFillPickerOpen || isTextColorPickerOpen) {
			// Add listener when either picker is open
			document.addEventListener('click', handleClickOutside);
		} else {
			// Remove listener when both pickers are closed
			document.removeEventListener('click', handleClickOutside);
		}
	}

	// Cleanup listener on component destroy
	onDestroy(() => {
		document.removeEventListener('click', handleClickOutside);
	});


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
		document.addEventListener('pointermove', handleDragMove); // Listen on document
		document.addEventListener('pointerup', handleDragEnd, { once: true });
		document.addEventListener('pointercancel', handleDragEnd, { once: true });
		document.body.style.cursor = 'grabbing'; // Indicate dragging
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
			(event.target as HTMLElement)?.releasePointerCapture?.(event.pointerId);
		} catch(e) { /* ignore */ }

		document.removeEventListener('pointermove', handleDragMove);
		// pointerup/cancel listeners were added to document with { once: true }
		document.body.style.cursor = ''; // Reset cursor
	}

	// --- Attribute Editing ---
	// Simple approach: local state per input, update on blur/enter
	let attributeEdits: Record<string, any> = {};

	function handleAttributeChange(key: string, value: any) {
		if (!selectedDataNode) return;
		// Update local temporary state first if needed, or directly call store action
		console.log(`Updating data attribute ${key} to:`, value);
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
			handleDragEnd(new PointerEvent('pointerup')); // Simulate pointer up for drag
		}
		// Cleanup resize listeners if component is destroyed while resizing
		if (isResizing && panelElement) {
			handleResizeEnd(new PointerEvent('pointerup')); // Simulate pointer up for resize
		}
	});

	// Action to measure header height
	function measureHeaderHeight(node: HTMLElement) {
		// Run after mount and updates
		const updateHeight = () => {
			if (node) {
				headerHeight = node.offsetHeight;
				// console.log('Header height measured:', headerHeight); // Optional debug log
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
		document.addEventListener('pointermove', handleResizeMove);
		document.addEventListener('pointerup', handleResizeEnd, { once: true }); // Use once for cleanup
		document.addEventListener('pointercancel', handleResizeEnd, { once: true });

		// Set cursor based on direction
		switch (direction) {
			case 'left':
			case 'right':
				document.body.style.cursor = 'ew-resize';
				break;
			case 'bottom':
				document.body.style.cursor = 'ns-resize';
				break;
			case 'bottom-left':
				document.body.style.cursor = 'nesw-resize'; // Note: opposite corner for diagonal
				break;
			case 'bottom-right':
				document.body.style.cursor = 'nwse-resize';
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
		if (resizeDirection.includes('left')) {
			newWidth = panelInitialWidth - dx;
			if (newWidth >= MIN_PANEL_WIDTH) {
				newX = panelInitialResizeX + dx;
			} else {
				newWidth = MIN_PANEL_WIDTH; // Prevent shrinking beyond min and moving left edge
				newX = panelInitialResizeX + (panelInitialWidth - MIN_PANEL_WIDTH);
			}
		} else if (resizeDirection.includes('right')) {
			newWidth = panelInitialWidth + dx;
		}

		if (resizeDirection.includes('bottom')) {
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
		if (newY < 0) { // Should not happen with current handles, but good practice
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
			(event.target as HTMLElement)?.releasePointerCapture?.(event.pointerId);
		} catch (e) {
			// Ignore errors, capture might already be released or element gone
		}

		isResizing = false;
		resizeDirection = null;
		document.removeEventListener('pointermove', handleResizeMove);
		// 'pointerup' and 'pointercancel' listeners were added to document with { once: true }
		document.body.style.cursor = ''; // Reset cursor
		// Check if the pointer capture target exists before trying to release
		// This handles cases where the element might be removed during the operation
		if (document.pointerLockElement) {
			try {
				document.exitPointerLock(); // General cleanup
			} catch (e) { /* ignore */ }
		}
		// Attempt to release capture specifically if possible, might error if element removed
		try {
			(event.target as HTMLElement)?.releasePointerCapture?.(event.pointerId);
		} catch(e) { /* ignore */ }

		isResizing = false;
		resizeDirection = null;
		document.removeEventListener('pointermove', handleResizeMove);
		document.body.style.cursor = ''; // Reset cursor
	}

	/*
	Future Considerations:
	- Multiple Panels: Refactor to use props and manage state centrally.
	- Locking: Add lock state.
	- Input Components: Create dedicated input components.
	- Debouncing/Saving Strategy: Refine attribute saving.
	*/
</script>

{#if $propertiesPanelVisible && selectedDataNode}
	<div
		bind:this={panelElement}
		class="properties-panel absolute flex flex-col bg-gray-100 dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg shadow-xl z-40 text-gray-900 dark:text-gray-100 overflow-hidden"
		style:height={$propertiesPanelCollapsed ? (headerHeight > 0 ? `${headerHeight}px` : '40px') : `${$propertiesPanelSize.height}px`}
		style:left="{$propertiesPanelPosition.x}px"
		style:top="{$propertiesPanelPosition.y}px"
		style:width="{$propertiesPanelSize.width}px"
		aria-labelledby="properties-panel-title"
	>
		<!-- Header -->
		<div
			bind:this={headerElement}
			class="panel-header flex items-center justify-between p-2 border-b border-gray-300 dark:border-gray-600 bg-gray-200 dark:bg-gray-700 rounded-t-lg select-none"
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
					on:click={() => { console.log('Collapse button clicked!'); togglePropertiesPanelCollapsed(); }}
					class="p-1 rounded hover:bg-gray-300 dark:hover:bg-gray-600"
					aria-label={$propertiesPanelCollapsed ? 'Expand Panel' : 'Collapse Panel'}
					title={$propertiesPanelCollapsed ? 'Expand Panel' : 'Collapse Panel'}
				>
					<Minimize2 size={14} />
				</button>
			</div>
		</div>

		<!-- Content (Scrollable) -->
		{#if !$propertiesPanelCollapsed} <!-- Use #if block to conditionally render content -->
		<div
			class="panel-content flex-grow p-3 overflow-y-auto space-y-4 [&::-webkit-scrollbar]:w-2 [&::-webkit-scrollbar-track]:bg-transparent [&::-webkit-scrollbar-thumb]:bg-gray-600 [&::-webkit-scrollbar-thumb]:rounded-full"
		>
				<!-- Attributes Section -->
				<section>
					<h3 class="text-xs font-semibold uppercase text-gray-500 dark:text-gray-400 mb-2">
						Attributes
					</h3>
					<div class="space-y-2">
						{#each Object.entries(selectedDataNode.attributes) as [key, value]}
							<!-- Only show attributes NOT defined in the type-specific schema, and not internal flags or view defaults, and not the old fontSize -->
							{#if key !== 'isSystemNode' && key !== 'fontSize' && !key.startsWith('karta_') && !typeSpecificKeys.has(key)}
								<div class="flex items-center justify-between gap-2">
									<label for="attr-{key}" class="text-sm capitalize truncate">{key}</label>
									{#if key === 'name'}
										{#if selectedDataNode.attributes.isSystemNode}
											<!-- Read-only display for system node names -->
											<span class="text-sm text-gray-600 dark:text-gray-300 truncate px-2 py-1">
												{value ?? ''}
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
											{typeof value === 'object' ? JSON.stringify(value) : (value ?? '')}
										</span>
									{/if}
								</div>
							{/if}
						{/each}
					</div>
				</section>

				<!-- Type Properties Section Removed -->

				<!-- Text Node View Properties Section -->
				{#if selectedDataNode.ntype === 'text' && selectedViewNode}
				{#key selectedViewNode} <!-- Force re-render when selectedViewNode reference changes -->
				<section>
					<h3 class="text-xs font-semibold uppercase text-gray-500 dark:text-gray-400 mb-2">
						Text View Styles (Context Specific)
					</h3>
					<div class="space-y-2">
						<!-- Fill Color -->
						<div class="flex items-center justify-between gap-2 relative">
							<label for="view-fillColor" class="text-sm">Fill Color</label>
								<ColorPicker
									bind:rgb={fillColorRgb}
									onInput={(e) => handleColorChange('karta_fillColor', e)}
									bind:isOpen={isFillPickerOpen}
									position="responsive"
								/>
						</div>
						<!-- Text Color -->
						<div class="flex items-center justify-between gap-2 relative">
							<label for="view-textColor" class="text-sm">Text Color</label>
							<!-- Color Swatch to open picker -->
								<ColorPicker
									bind:rgb={textColorRgb}
									onInput={(e) => handleColorChange('karta_textColor', e)}
									bind:isOpen={isTextColorPickerOpen}
									position="responsive"
								/>
						</div>
						<!-- Font -->
						<div class="flex items-center justify-between gap-2">
							<label for="view-font" class="text-sm">Font</label>
							<select
								id="view-font"
								bind:value={selectedFontValue}
								on:change={handleFontChange}
								class="w-full px-2 py-1 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 text-sm"
							>
								{#each AVAILABLE_FONTS as font}
									<option value={font}>{font}</option>
								{/each}
							</select>
						</div>
						<!-- Font Size -->
						<div class="flex items-center justify-between gap-2">
							<label for="view-fontSize" class="text-sm">Font Size</label>
							<input
								type="number"
								id="view-fontSize"
								bind:value={selectedFontSizeValue}
								on:change={(e) => {
									if (selectedViewNode) {
										updateViewNodeAttribute(selectedViewNode.id, 'karta_fontSize', parseFloat((e.target as HTMLInputElement).value) || FALLBACK_FONT_SIZE);
									}
								}}
								class="w-20 px-2 py-1 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 text-sm"
							/>
						</div>
					</div>
				</section>
				{/key} <!-- End of #key block -->
				{/if} <!-- End of Text Node View Properties Section #if -->

			</div> <!-- End of panel-content div -->
		{/if} <!-- End of #if !$propertiesPanelCollapsed -->
<!-- Invisible Resize Handles -->
{#if !$propertiesPanelCollapsed}
	<!-- Left -->
	<div data-direction="left" class="absolute top-0 left-0 h-full cursor-ew-resize" style="width: {HANDLE_SIZE}px;" on:pointerdown|stopPropagation={handleResizeStart}></div>
	<!-- Right -->
	<div data-direction="right" class="absolute top-0 right-0 h-full cursor-ew-resize" style="width: {HANDLE_SIZE}px;" on:pointerdown|stopPropagation={handleResizeStart}></div>
	<!-- Bottom -->
	<div data-direction="bottom" class="absolute bottom-0 left-0 w-full cursor-ns-resize" style="height: {HANDLE_SIZE}px;" on:pointerdown|stopPropagation={handleResizeStart}></div>
	<!-- Bottom-Left Corner -->
	<div data-direction="bottom-left" class="absolute bottom-0 left-0 cursor-nesw-resize" style="width: {HANDLE_SIZE * 2}px; height: {HANDLE_SIZE * 2}px;" on:pointerdown|stopPropagation={handleResizeStart}></div>
	<!-- Bottom-Right Corner -->
	<div data-direction="bottom-right" class="absolute bottom-0 right-0 cursor-nwse-resize" style="width: {HANDLE_SIZE * 2}px; height: {HANDLE_SIZE * 2}px;" on:pointerdown|stopPropagation={handleResizeStart}></div>
{/if}

</div> <!-- End of main properties-panel div -->
{/if}

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