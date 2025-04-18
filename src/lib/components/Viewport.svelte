<!--
// --- Karta Runtime Component ---
// This file is planned for inclusion in the MIT-licensed `karta_runtime` package.
// It should focus on displaying the graph and handling interactions in "Play Mode".
// Avoid adding editor-specific logic or dependencies here.
// Interaction logic should read configuration from node attributes.
-->
<script lang="ts">
	import { get } from 'svelte/store';
	import { onMount, onDestroy } from 'svelte'; // Added onDestroy
	import {
		viewTransform, // Keep the Tween object
		screenToCanvasCoordinates,
		nodes,
		contexts,
		currentContextId,
		currentTool,
		cancelConnectionProcess,
		// Create Node Menu
		isCreateNodeMenuOpen,
		createNodeMenuPosition,
		openCreateNodeMenu,
		closeCreateNodeMenu,
		// Context Menu
		isContextMenuOpen,
		contextMenuPosition,
		contextMenuContext,
		openContextMenu,
		closeContextMenu,
		type ContextMenuContextType,
		switchContext, // Action for 'Enter Context'
		centerViewOnCanvasPoint, // Action for 'Center View' (used by centerOnFocalNode)
		centerOnFocalNode, // Action for shortcut 'F'
		frameContext, // <-- Import action for menu/shortcut 'Shift+F'
		requestNodeRename, // Action for 'Rename'
		// Selection
		selectedNodeIds,
		clearSelection,
		setSelectedNodes,
		toggleSelection,
		currentViewNodes, // Store for node states in current context
		// Paste/Drop
		createTextNodeFromPaste,
		createImageNodeWithAsset
	} from '$lib/karta/KartaStore';
	import NodeWrapper from './NodeWrapper.svelte';
	import EdgeLayer from './EdgeLayer.svelte';
	import CreateNodeMenu from './CreateNodeMenu.svelte'; // Import the menu component
	import ContextMenu from './ContextMenu.svelte'; // Import the context menu component
	import SelectionBox from './SelectionBox.svelte'; // Import the selection box component
	// Removed Toolbar and ContextPathDisplay imports

	let canvasContainer: HTMLElement;
	let canvas: HTMLElement;

    // Reactive definition for the current context object
    $: currentCtx = $contexts.get($currentContextId);

    // Debug log for selection size
    $: console.log('[Viewport] Selected nodes count:', $selectedNodeIds.size);

	// State for panning
	let isPanning = false;
	let panStartX = 0;
	let panStartY = 0;
    let lastInputWasTouchpad = false; // State for heuristic

    // State for last known cursor position
    let lastScreenX = 0;
    let lastScreenY = 0;

    // State for marquee selection
    let isMarqueeSelecting = false;
    let marqueeStartCoords: { canvasX: number; canvasY: number } | null = null;
    let marqueeEndCoords: { canvasX: number; canvasY: number } | null = null;
    let initialSelection: Set<string> | null = null; // Store selection at drag start
    let marqueeRectElement: HTMLDivElement | null = null; // For the visual element

	let contextMenuElement: HTMLElement | null = null; // Reference to the context menu div

	// --- Constants for Scale Invariance ---
	const desiredScreenOutlineWidth = 1; // Target outline width in screen pixels

	// --- REMOVED Reactive Calculations for SelectionBox Props ---
	// $: currentScale = $currentViewTransform?.scale ?? 1; // REMOVED
	// $: invScale = currentScale > 0 ? 1 / currentScale : 1; // REMOVED
	// $: outlineWidth = currentScale > 0 ? desiredScreenOutlineWidth / currentScale : desiredScreenOutlineWidth; // REMOVED

	// --- Helper Functions ---
	// Helper function to get image dimensions from an Object URL or Data URL
	function getImageDimensionsFromUrl(url: string): Promise<{ width: number; height: number }> {
		return new Promise((resolve, reject) => {
			const img = new Image();
			img.onload = () => {
				resolve({ width: img.naturalWidth, height: img.naturalHeight });
			};
			img.onerror = (error) => {
				console.error('[Viewport] Error loading image to get dimensions:', error);
				// Fallback to default dimensions if loading fails
				resolve({ width: 100, height: 100 }); // Or use registry defaults?
			};
			img.src = url;
		});
	}

	// --- Event Handlers ---

// --- Event Handlers ---
	function handleWheel(e: WheelEvent) {
    // console.log(`handleWheel: deltaY=${e.deltaY}, deltaX=${e.deltaX}, deltaMode=${e.deltaMode}, ctrlKey=${e.ctrlKey}`); // DEBUG LOG removed
    e.preventDefault();
    if (!canvasContainer) return;

    const rect = canvasContainer.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    const currentTransform = viewTransform.target;
    const beforeZoomX = (mouseX - currentTransform.posX) / currentTransform.scale;
    const beforeZoomY = (mouseY - currentTransform.posY) / currentTransform.scale;

    let newScale = currentTransform.scale;
    let newPosX = currentTransform.posX;
    let newPosY = currentTransform.posY;
    const zoomSensitivityFactor = 0.5; // Slightly slower zoom
    const panSensitivityFactor = 1.6;  // Slightly faster pan
    const wheelZoomFactor = 1.75; // Increased Standard wheel zoom factor
    const pinchZoomSensitivity = 0.09; // Touchpad pinch zoom sensitivity

    // --- Heuristic Update ---
    // Detect if input is likely touchpad pan (both X and Y deltas present)
    const deltaThreshold = 0.1; // Use a small threshold
    if (Math.abs(e.deltaX) > deltaThreshold && Math.abs(e.deltaY) > deltaThreshold) {
        lastInputWasTouchpad = true;
    }

    if (e.ctrlKey) {
        // Pinch-to-zoom (Ctrl key pressed) - Always zoom
        const pinchFactor = 1 + pinchZoomSensitivity * zoomSensitivityFactor;
        newScale = currentTransform.scale * (e.deltaY < 0 ? pinchFactor : 1 / pinchFactor);
        newScale = Math.max(0.1, Math.min(newScale, 5));
        newPosX = mouseX - beforeZoomX * newScale;
        newPosY = mouseY - beforeZoomY * newScale;
    } else if (lastInputWasTouchpad) {
        // Touchpad panning (heuristic detected touchpad)
        newPosX = currentTransform.posX - e.deltaX * panSensitivityFactor;
        newPosY = currentTransform.posY - e.deltaY * panSensitivityFactor;
        // Keep scale the same when panning
        newScale = currentTransform.scale;
    } else {
        // Standard mouse wheel zoom (heuristic assumes mouse)
        newScale = currentTransform.scale * (e.deltaY < 0 ? wheelZoomFactor : 1 / wheelZoomFactor);
        newScale = Math.max(0.1, Math.min(newScale, 5));
        newPosX = mouseX - beforeZoomX * newScale;
        newPosY = mouseY - beforeZoomY * newScale;
    }

    // Close menus if transform changes
    if (newScale !== currentTransform.scale || newPosX !== currentTransform.posX || newPosY !== currentTransform.posY) {
        closeContextMenu();
        closeCreateNodeMenu();
    }
    viewTransform.set({ scale: newScale, posX: newPosX, posY: newPosY }, {duration: 140});

    // Call tool's wheel handler
    get(currentTool)?.onWheel?.(e);
}

	function handlePointerDown(e: PointerEvent) {
		// Middle mouse panning takes precedence
		if (e.button === 1) {
			lastInputWasTouchpad = false; // Middle mouse click means it's definitely a mouse
			e.preventDefault();
			isPanning = true;
			const currentTransform = viewTransform.target; // Use target for initial calculation
			panStartX = e.clientX - currentTransform.posX;
			panStartY = e.clientY - currentTransform.posY;
			// Ensure canvasContainer is bound before manipulating style/listeners
			if (canvasContainer) {
				canvasContainer.style.cursor = 'grabbing';
				// Capture the pointer for this drag sequence
				canvasContainer.setPointerCapture(e.pointerId);
				// Add listeners directly to the element that captured the pointer
				canvasContainer.addEventListener('pointermove', handleElementPointerMove);
				canvasContainer.addEventListener('pointerup', handleElementPointerUp);
			} else {
				console.error("handlePointerDown: canvasContainer not bound yet!");
			}
			return; // Don't delegate middle mouse
		}

		// Check if the click target is the background (canvas or container)
		const targetElement = e.target as HTMLElement;
		const clickedOnNode = targetElement.closest('.node-wrapper');
		const clickedOnBackground = targetElement === canvasContainer || targetElement === canvas;

		if (clickedOnBackground && e.button === 0) { // Left click on background
			e.preventDefault(); // Prevent default text selection/drag behavior
			isMarqueeSelecting = true;
			lastInputWasTouchpad = false; // Assume mouse interaction for marquee

			// Calculate start coords
			const rect = canvasContainer.getBoundingClientRect();
			const { x: canvasX, y: canvasY } = screenToCanvasCoordinates(e.clientX, e.clientY, rect);
			marqueeStartCoords = { canvasX, canvasY };
			marqueeEndCoords = { canvasX, canvasY }; // Initialize end coords
			initialSelection = new Set(get(selectedNodeIds)); // Store initial selection

			// Capture pointer on the container
			canvasContainer.setPointerCapture(e.pointerId);
			// Add move/up listeners specifically for marquee drag
			canvasContainer.addEventListener('pointermove', handleMarqueePointerMove);
			canvasContainer.addEventListener('pointerup', handleMarqueePointerUp);

			// Do NOT delegate to tool if starting marquee
			return;
		}

		// Need new handlers for marquee move/up, separate from general viewport move/up
		function handleMarqueePointerMove(e: PointerEvent) {
			if (!isMarqueeSelecting || !marqueeStartCoords || !canvasContainer) return;

			const rect = canvasContainer.getBoundingClientRect();
			const { x: currentCanvasX, y: currentCanvasY } = screenToCanvasCoordinates(e.clientX, e.clientY, rect);
			marqueeEndCoords = { canvasX: currentCanvasX, canvasY: currentCanvasY };

			// --- Update Selection Logic ---
			updateSelectionFromMarquee(e.shiftKey, e.ctrlKey || e.metaKey);

			// --- Update Visual Marquee ---
			updateMarqueeVisual(); // We'll define this helper later
		}

		function handleMarqueePointerUp(e: PointerEvent) {
			if (!isMarqueeSelecting || !canvasContainer) return;

			// Check if it was a click (minimal movement) vs a drag
			const dx = marqueeEndCoords ? Math.abs(marqueeEndCoords.canvasX - marqueeStartCoords!.canvasX) : 0;
			const dy = marqueeEndCoords ? Math.abs(marqueeEndCoords.canvasY - marqueeStartCoords!.canvasY) : 0;
			const dragThreshold = 5 / viewTransform.current.scale; // Adjust threshold based on zoom

			if (dx < dragThreshold && dy < dragThreshold) {
				// Treat as a click on the background
				clearSelection();
			} else {
				// Final selection was already set during move, just cleanup
			}

			// Cleanup
			canvasContainer.releasePointerCapture(e.pointerId);
			canvasContainer.removeEventListener('pointermove', handleMarqueePointerMove);
			canvasContainer.removeEventListener('pointerup', handleMarqueePointerUp);

			isMarqueeSelecting = false;
			marqueeStartCoords = null;
			marqueeEndCoords = null;
			initialSelection = null;
			if (marqueeRectElement) {
				marqueeRectElement.remove(); // Remove visual element
				marqueeRectElement = null;
			}
		}

		// Helper function to calculate and update selection during marquee
		function updateSelectionFromMarquee(shiftKey: boolean, ctrlOrMetaKey: boolean) {
			if (!isMarqueeSelecting || !marqueeStartCoords || !marqueeEndCoords || !canvasContainer || !initialSelection) return;

			const currentTransform = viewTransform.current;

			// Calculate marquee bounds in canvas coordinates
			const marqueeLeft = Math.min(marqueeStartCoords.canvasX, marqueeEndCoords.canvasX);
			const marqueeRight = Math.max(marqueeStartCoords.canvasX, marqueeEndCoords.canvasX);
			const marqueeTop = Math.min(marqueeStartCoords.canvasY, marqueeEndCoords.canvasY);
			const marqueeBottom = Math.max(marqueeStartCoords.canvasY, marqueeEndCoords.canvasY);

			const currentIntersectingIds = new Set<string>();
			const nodeElements = canvas?.querySelectorAll<HTMLElement>('.node-wrapper'); // Query within canvas

			nodeElements?.forEach(nodeEl => {
				const nodeId = nodeEl.dataset.id;
				if (!nodeId) return;

				// Get node bounds (more accurate than viewNode state if possible)
				// This uses screen coords, needs conversion
				const nodeRect = nodeEl.getBoundingClientRect();
				const viewportRect = canvasContainer.getBoundingClientRect();

				// Convert node screen bounds to canvas bounds
				const nodeCanvasTopLeft = screenToCanvasCoordinates(nodeRect.left, nodeRect.top, viewportRect);
				const nodeCanvasBottomRight = screenToCanvasCoordinates(nodeRect.right, nodeRect.bottom, viewportRect);

				// Simple AABB intersection check
				if (
					nodeCanvasTopLeft.x < marqueeRight &&
					nodeCanvasBottomRight.x > marqueeLeft &&
					nodeCanvasTopLeft.y < marqueeBottom &&
					nodeCanvasBottomRight.y > marqueeTop
				) {
					currentIntersectingIds.add(nodeId);
				}
			});

			// Determine target selection based on modifiers and initial state
			let targetSelection: Set<string>;
			if (shiftKey) {
				targetSelection = new Set([...initialSelection, ...currentIntersectingIds]);
			} else if (ctrlOrMetaKey) {
				targetSelection = new Set([...initialSelection].filter(id => !currentIntersectingIds.has(id)));
			} else {
				targetSelection = currentIntersectingIds;
			}

			// Update the main store directly
			selectedNodeIds.set(targetSelection); // Note: This might trigger many updates
		}

		// Creates or updates the visual marquee rectangle element
		function updateMarqueeVisual() {
			if (!isMarqueeSelecting || !marqueeStartCoords || !marqueeEndCoords || !canvasContainer) return;

			const transform = viewTransform.current;

			// Convert canvas coords to screen coords
			const screenStartX = marqueeStartCoords.canvasX * transform.scale + transform.posX;
			const screenStartY = marqueeStartCoords.canvasY * transform.scale + transform.posY;
			const screenEndX = marqueeEndCoords.canvasX * transform.scale + transform.posX;
			const screenEndY = marqueeEndCoords.canvasY * transform.scale + transform.posY;

			const left = Math.min(screenStartX, screenEndX);
			const top = Math.min(screenStartY, screenEndY);
			const width = Math.abs(screenStartX - screenEndX);
			const height = Math.abs(screenStartY - screenEndY);

			if (!marqueeRectElement) {
				// Create the element if it doesn't exist
				marqueeRectElement = document.createElement('div');
				marqueeRectElement.style.position = 'absolute';
				marqueeRectElement.style.border = '1px dashed #cbd5e1'; // Tailwind slate-300
				marqueeRectElement.style.backgroundColor = 'rgba(59, 130, 246, 0.1)'; // Tailwind blue-500 with opacity
				marqueeRectElement.style.pointerEvents = 'none'; // Ignore pointer events
				marqueeRectElement.style.zIndex = '50'; // Ensure it's above nodes/edges
				marqueeRectElement.setAttribute('aria-hidden', 'true');
				canvasContainer.appendChild(marqueeRectElement);
			}

			// Update position and size
			marqueeRectElement.style.left = `${left}px`;
			marqueeRectElement.style.top = `${top}px`;
			marqueeRectElement.style.width = `${width}px`;
			marqueeRectElement.style.height = `${height}px`;
		}

		// If not middle mouse or marquee start, delegate to the active tool
		// Pass the event and the direct target element
		get(currentTool)?.onPointerDown?.(e, e.target as EventTarget | null);
	}

    // New handler for pointer move on the element during middle-mouse pan
    function handleElementPointerMove(e: PointerEvent) {
        // Check if we are still panning (redundant check if listeners are removed correctly, but safe)
        // Also check if the moving pointer is the one we captured
        // Add null check for canvasContainer
        if (isPanning && canvasContainer && canvasContainer.hasPointerCapture(e.pointerId)) {
            const newPosX = e.clientX - panStartX;
			const newPosY = e.clientY - panStartY;
            // Close menus if transform changes
            if (newPosX !== viewTransform.target.posX || newPosY !== viewTransform.target.posY) {
                closeContextMenu();
                closeCreateNodeMenu();
            }
            viewTransform.set({ scale: viewTransform.target.scale, posX: newPosX, posY: newPosY }, { duration: 0 });
        }
    }

    // New handler for pointer up on the element during middle-mouse pan
    function handleElementPointerUp(e: PointerEvent) {
        // Check if this is the up event for the pointer we captured and the middle button
        // Add null check for canvasContainer
        if (isPanning && e.button === 1 && canvasContainer && canvasContainer.hasPointerCapture(e.pointerId)) {
            isPanning = false;
            // No need for inner null check now, already checked above
            canvasContainer.style.cursor = 'default'; // Reset cursor
            // Remove listeners from the element
            canvasContainer.removeEventListener('pointermove', handleElementPointerMove);
            canvasContainer.removeEventListener('pointerup', handleElementPointerUp);
            // Release the pointer capture
            canvasContainer.releasePointerCapture(e.pointerId);
        }
    }

	// General pointer move on viewport (for non-panning moves, delegate to tool)
	function handleViewportPointerMove(e: PointerEvent) { // Changed to PointerEvent
        // Update last known cursor position
        lastScreenX = e.clientX;
        lastScreenY = e.clientY;

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
	// Check if focus is currently within an input, textarea, or contenteditable element
	const activeEl = document.activeElement;
	const isInputFocused = activeEl && (
		activeEl.tagName === 'INPUT' ||
		activeEl.tagName === 'TEXTAREA' ||
		(activeEl instanceof HTMLElement && activeEl.isContentEditable)
	);

	if (e.key === 'Tab') {
		// Open Create Node Menu, but only if an input isn't focused.
		// Default focus cycling is prevented globally in +layout.svelte.
		if (!isInputFocused) {
			if (!canvasContainer) return;

			const rect = canvasContainer.getBoundingClientRect();
			let screenX = lastScreenX;
			let screenY = lastScreenY;

			// Fallback to center if cursor hasn't moved over viewport yet
			if (screenX === 0 && screenY === 0) {
				screenX = rect.left + rect.width / 2;
				screenY = rect.top + rect.height / 2;
				console.log('Tab pressed: Cursor position unknown, using viewport center.');
			} else {
				console.log(`Tab pressed: Using cursor position (${screenX}, ${screenY})`);
			}

			// Calculate canvas coordinates
			const { x: canvasX, y: canvasY } = screenToCanvasCoordinates(screenX, screenY, rect);
			console.log(`Tab pressed: Calculated canvas coordinates (${canvasX}, ${canvasY})`);

			// Open the menu
			openCreateNodeMenu(screenX, screenY, canvasX, canvasY);
			// DO NOT call e.preventDefault() here - it's handled globally
		} else {
			console.log('[Viewport] Tab ignored for menu (input focused).');
		}

	} else if (e.key === 'f' || e.key === 'F') {
		// Handle both 'F' and 'Shift+F'
		if (e.shiftKey && !e.ctrlKey && !e.metaKey && !e.altKey) {
			// Shift+F: Frame Context
			e.preventDefault();
			frameContext();
		} else if (!e.shiftKey && !e.ctrlKey && !e.metaKey && !e.altKey) {
			// F (no modifiers): Center Focal Node
			e.preventDefault();
			centerOnFocalNode();
		}
		// Ignore if other modifiers are pressed (e.g., Ctrl+F for browser search)
	}

	// Delegate keydown events to the active tool (unless handled above)
	if (!e.defaultPrevented) {
		get(currentTool)?.onKeyDown?.(e);
	}


		      // Keep Escape key handling here for global cancel? Or move to tool?
		      // Let's keep it global for now.
		      if (e.key === 'Escape') {
            cancelConnectionProcess();
            closeCreateNodeMenu(); // Also close create menu on Escape
            closeContextMenu(); // Also close context menu on Escape
           }
          }

    function handleKeyUp(e: KeyboardEvent) {
        // Delegate keyup events to the active tool
        get(currentTool)?.onKeyUp?.(e);
	}

	function handleContextMenu(e: MouseEvent) {
		e.preventDefault(); // Prevent default browser context menu

		const targetElement = e.target as HTMLElement;
		const clickedNodeWrapper = targetElement.closest('.node-wrapper');
		const clickedEdge = targetElement.closest('svg .edge-path'); // Basic check for edge click

		let context: ContextMenuContextType;

		if (clickedNodeWrapper) {
			const nodeId = (clickedNodeWrapper as HTMLElement).dataset.id;
			context = { type: 'node', id: nodeId };
			console.log('Context menu on node:', nodeId);
		} else if (clickedEdge) {
			// TODO: Enhance EdgeLayer to add data-id to paths to identify specific edge
			const edgeId = (clickedEdge as HTMLElement).dataset.id; // Assuming data-id exists later
			context = { type: 'edge', id: edgeId || 'unknown' }; // Use unknown if ID not found yet
			console.log('Context menu on edge:', edgeId || 'unknown');
		} else {
			context = { type: 'background' };
			console.log('Context menu on background');
		}

		openContextMenu({ x: e.clientX, y: e.clientY }, context);
	}

	/** Closes the context menu if a click occurs outside of it. */
	function handleClickOutsideContextMenu(event: PointerEvent) {
		if ($isContextMenuOpen && contextMenuElement && !contextMenuElement.contains(event.target as Node)) {
			// Check if the click target is the menu itself or one of its descendants
			// Also check if the click target is the element that *opened* the menu (prevent immediate close)
			// For now, a simple check is sufficient. Refine if needed.
			closeContextMenu();
		}
	}

	// Removed handleDoubleClick

    onMount(() => {
        // Focus the viewport container when the component mounts
        // This helps ensure keyboard events are captured correctly.
        canvasContainer?.focus();

  // Global listener for closing context menu on outside click
  window.addEventListener('pointerdown', handleClickOutsideContextMenu);

  // Cleanup listener on component destroy
  return () => {
   window.removeEventListener('pointerdown', handleClickOutsideContextMenu);
  };
    });

// --- Paste/Drop Handlers ---

function handleDragOver(e: DragEvent) {
 e.preventDefault(); // Necessary to allow dropping
 // Optional: Add visual feedback (e.g., change border style)
 if (canvasContainer) {
  // Example: Add a class, remove it on dragleave/drop
 }
}

async function handleDrop(e: DragEvent) {
 e.preventDefault();
 if (!e.dataTransfer || !canvasContainer) return;

 console.log('[Viewport] Drop event detected.');
 const rect = canvasContainer.getBoundingClientRect();

 for (const item of e.dataTransfer.items) {
  if (item.kind === 'file' && item.type.startsWith('image/')) {
   const file = item.getAsFile();
   if (file) {
    console.log(`[Viewport] Dropped image file: ${file.name}`);
    try {
    		      // Create Object URL (must be revoked later if creation fails)
    		      const objectUrl = URL.createObjectURL(file);
    		      const assetName = file.name || 'Dropped Image';
    		      const { x: canvasX, y: canvasY } = screenToCanvasCoordinates(e.clientX, e.clientY, rect);
    		      // Get dimensions from the object URL
    		      const dimensions = await getImageDimensionsFromUrl(objectUrl);
    		      console.log(`[Viewport] Creating image node via asset at canvas coords: (${canvasX}, ${canvasY}) with dimensions ${dimensions.width}x${dimensions.height}`);
    		      // Call the new store action, passing the Blob, Object URL, and dimensions
    		      createImageNodeWithAsset({ x: canvasX, y: canvasY }, file, objectUrl, assetName, dimensions.width, dimensions.height);
    } catch (error) {
    	console.error('[Viewport] Error reading dropped file:', error);
    }
   }
  }
 }
 // Optional: Remove visual feedback added in handleDragOver
}

async function handlePaste(e: ClipboardEvent) {
 // Ignore paste events originating from inputs/textareas/contenteditables
 const target = e.target as HTMLElement;
 if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
  console.log('[Viewport] Paste event ignored (target is editable).');
  return;
 }

 if (!e.clipboardData || !canvasContainer) return;
 console.log('[Viewport] Paste event detected on viewport.');

 e.preventDefault(); // Handle paste ourselves

 const rect = canvasContainer.getBoundingClientRect();
 let pasteCanvasX: number;
 let pasteCanvasY: number;

 // Determine paste position (last cursor or center)
 if (lastScreenX !== 0 || lastScreenY !== 0) {
  const coords = screenToCanvasCoordinates(lastScreenX, lastScreenY, rect);
  pasteCanvasX = coords.x;
  pasteCanvasY = coords.y;
  console.log(`[Viewport] Using last cursor position for paste: (${pasteCanvasX}, ${pasteCanvasY})`);
 } else {
  const centerX = rect.left + rect.width / 2;
  const centerY = rect.top + rect.height / 2;
  const coords = screenToCanvasCoordinates(centerX, centerY, rect);
  pasteCanvasX = coords.x;
  pasteCanvasY = coords.y;
  console.log(`[Viewport] Using viewport center for paste: (${pasteCanvasX}, ${pasteCanvasY})`);
 }


 for (const item of e.clipboardData.items) {
  if (item.kind === 'file' && item.type.startsWith('image/')) {
   const file = item.getAsFile();
   if (file) {
    console.log(`[Viewport] Pasted image file: ${file.name}`);
   try {
            // Create Object URL (must be revoked later if creation fails)
            const objectUrl = URL.createObjectURL(file);
            const assetName = file.name || 'Pasted Image';
            // Get dimensions from the object URL
            const dimensions = await getImageDimensionsFromUrl(objectUrl);
            console.log(`[Viewport] Creating image node via asset from paste at canvas coords: (${pasteCanvasX}, ${pasteCanvasY}) with dimensions ${dimensions.width}x${dimensions.height}`);
            // Call the new store action, passing the Blob, Object URL, and dimensions
            createImageNodeWithAsset({ x: pasteCanvasX, y: pasteCanvasY }, file, objectUrl, assetName, dimensions.width, dimensions.height);
            return; // Handle first image found
    } catch (error) {
    	console.error('[Viewport] Error reading pasted file:', error);
    }
   }
  } else if (item.kind === 'string' && item.type === 'text/plain') {
   item.getAsString(text => {
    if (text && text.trim().length > 0) {
    	console.log(`[Viewport] Pasted text: "${text.substring(0, 50)}..."`);
    	console.log(`[Viewport] Creating text node from paste at canvas coords: (${pasteCanvasX}, ${pasteCanvasY})`);
    	createTextNodeFromPaste({ x: pasteCanvasX, y: pasteCanvasY }, text);
    	return; // Handle first text found
    }
  });
 } // End of for loop
 } // End of handlePaste function
} // End of handlePaste function

// Removed duplicate helper function readFileAsDataURL

</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
id="viewport"
class="karta-viewport-container w-full h-screen overflow-hidden relative cursor-default bg-gray-800"
	bind:this={canvasContainer}
	on:pointerdown={handlePointerDown}
	on:pointermove={handleViewportPointerMove}
	on:pointerup={handleViewportPointerUp}
	on:wheel={handleWheel}
	on:contextmenu={handleContextMenu}
	tabindex="0"
	on:keydown={handleKeyDown}
	on:keyup={handleKeyUp}
	on:paste={handlePaste}
	on:dragover={handleDragOver}
	on:drop={handleDrop}
>
	<div
		class="w-full h-full relative origin-top-left"
		bind:this={canvas}
		style:transform="translate({viewTransform.current.posX}px, {viewTransform.current.posY}px) scale({viewTransform.current.scale})"
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
		<!-- Selection Box (now always mounted, internal logic handles visibility) - Moved INSIDE transformed canvas -->
		{#if true} <!-- Wrap in valid block to fix {@const} placement -->
			{@const currentScaleValue = viewTransform.current.scale}
			{@const invScaleValue = currentScaleValue > 0 ? 1 / currentScaleValue : 1}
			{@const outlineWidthValue = currentScaleValue > 0 ? desiredScreenOutlineWidth / currentScaleValue : desiredScreenOutlineWidth}
			<SelectionBox inverseScale={invScaleValue} canvasOutlineWidth={outlineWidthValue} />
		{/if}
	</div>

	<!-- Create Node Menu (conditionally rendered) -->
	{#if $isCreateNodeMenuOpen && $createNodeMenuPosition}
		<!-- Create Node Menu related elements -->
		{@const transform = viewTransform.current} <!-- Access tween value directly -->
		{@const markerScreenX = $createNodeMenuPosition.canvasX * transform.scale + transform.posX}
		{@const markerScreenY = $createNodeMenuPosition.canvasY * transform.scale + transform.posY}

		<!-- Position Marker (positioned using transformed canvas coords) -->
		<div
			class="absolute w-3 h-3 border-2 border-blue-400 rounded-full bg-blue-400 bg-opacity-30 pointer-events-none z-40"
			style:left="{markerScreenX - 6}px"
			style:top="{markerScreenY - 6}px"
			aria-hidden="true"
		></div>

		<!-- The Menu Component (positioned using screen coords) -->
		<CreateNodeMenu x={$createNodeMenuPosition.screenX + 10} y={$createNodeMenuPosition.screenY + 10} />
	{/if}

	<!-- Create Node Menu elements remain outside the transformed canvas, positioned relative to viewport -->
</div>

	<!-- Context Menu (conditionally rendered) -->
	{#if $isContextMenuOpen && $contextMenuPosition}
		{@const contextType = $contextMenuContext?.type}
		{@const targetNodeId = $contextMenuContext?.id}
		{@const currentNodesMap = $nodes}
		{@const currentViewNodesMap = $currentViewNodes}

		<!-- Dynamically generate menu items based on context -->
		{@const menuItems = (() => {
			let items: { label: string; action: () => void; disabled?: boolean }[] = [];
			const targetDataNode = targetNodeId ? currentNodesMap.get(targetNodeId) : null;
			const targetViewNode = targetNodeId ? currentViewNodesMap.get(targetNodeId) : null;
			const screenPos = $contextMenuPosition; // Has screenX/Y

			// Helper to get canvas coords from stored screen coords
			const getCanvasCoords = () => {
				if (!screenPos || !canvasContainer) return { x: 0, y: 0 };
				const rect = canvasContainer.getBoundingClientRect();
				// Use screenPos.x and screenPos.y which are the correct screen coordinates
				return screenToCanvasCoordinates(screenPos.x, screenPos.y, rect);
			};

			if (contextType === 'node' && targetNodeId) {
				const nodeState = targetViewNode?.state.current;
				items = [
					{
						label: 'Enter Context',
						action: () => { if (targetNodeId) switchContext(targetNodeId); },
						disabled: !targetNodeId || targetNodeId === $currentContextId
					},
					{
						label: 'Center View',
						action: () => {
							if (nodeState) {
								centerViewOnCanvasPoint(nodeState.x + nodeState.width / 2, nodeState.y + nodeState.height / 2);
							}
						},
						disabled: !nodeState
					},
					{
						label: 'Rename',
						action: () => { if (targetNodeId) requestNodeRename(targetNodeId); },
						disabled: !targetDataNode || targetDataNode.attributes?.isSystemNode
					},
					// Add Delete options later
				];
			} else if (contextType === 'edge' && targetNodeId) {
				items = [
					{ label: 'Delete Edge', action: () => console.warn('Delete Edge NYI'), disabled: true }
				];
			} else if (contextType === 'background') {
				items = [
					{
						label: 'Center Focal Node',
						action: () => centerOnFocalNode(),
						disabled: !$currentContextId // Disable if no context (shouldn't happen?)
					},
					{
						label: 'Frame Context',
						action: () => frameContext(), // Action to be implemented in KartaStore
						disabled: !$currentContextId
					},
					{
						label: 'Create Node Here',
						action: () => {
							const { x: canvasX, y: canvasY } = getCanvasCoords();
							// Use screenPos.x and screenPos.y for the menu positioning part
							openCreateNodeMenu(screenPos!.x, screenPos!.y, canvasX, canvasY);
						},
						disabled: !screenPos
					},
					{ label: 'Paste', action: () => console.warn('Paste action NYI from context menu.'), disabled: true }
				];
			}
			return items;
		})()}

		<div bind:this={contextMenuElement}>
			<!-- Pass only screen coordinates for positioning -->
			<ContextMenu position={$contextMenuPosition} items={menuItems} />
		</div>
	{/if}
