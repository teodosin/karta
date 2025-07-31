<!--
// --- Karta Runtime Component ---
// This file is planned for inclusion in the MIT-licensed `karta_runtime` package.
// It should focus on displaying the graph and handling interactions in "Play Mode".
// Avoid adding editor-specific logic or dependencies here.
// Interaction logic should read configuration from node attributes.
-->
<script lang="ts">
	import { get } from "svelte/store";
	import { onMount, onDestroy } from "svelte";
	import {
		contexts,
		currentContextId,
		currentViewNodes,
		switchContext,
		removeViewNodeFromContext,
	} from "$lib/karta/ContextStore";
	import {
		viewTransform,
		screenToCanvasCoordinates,
		centerViewOnCanvasPoint,
		centerOnFocalNode,
		frameContext,
		viewportWidth,
		viewportHeight,
	} from "$lib/karta/ViewportStore";
	import {
		currentTool,
		cancelConnectionProcess,
		isConnecting,
		updateTempLinePosition,
		finishConnectionProcess,
		isReconnecting,
		finishReconnectionProcess,
		startReconnectionProcess
	} from '$lib/karta/ToolStore';
	import {
		isCreateNodeMenuOpen,
		createNodeMenuPosition,
		openCreateNodeMenu,
		closeCreateNodeMenu,
		isFilterMenuOpen,
		filterMenuPosition,
		closeFilterMenu,
		openContextMenu,
		closeContextMenu,
		type ContextMenuContextType,
	} from "$lib/karta/UIStateStore";
	import {
		selectedNodeIds,
		clearSelection,
		setSelectedNodes,
		toggleSelection,
	} from "$lib/karta/SelectionStore";
	import {
		selectedEdgeIds,
		clearEdgeSelection,
		setSelectedEdges,
		toggleEdgeSelection,
	} from "$lib/karta/EdgeSelectionStore";
	import {
		nodes,
		createTextNodeFromPaste,
		createImageNodeWithAsset,
		findPhysicalParentPath,
	} from "$lib/karta/NodeStore";
	import { notifications } from '$lib/karta/NotificationStore';
	import { edges, deleteEdges } from "$lib/karta/EdgeStore";
	import NodeWrapper from "./NodeWrapper.svelte";
	import EdgeLayer from "./EdgeLayer.svelte";
	import CreateNodeMenu from "./CreateNodeMenu.svelte";
	import FilterMenuDropdown from "./FilterMenuDropdown.svelte";
	import ContextMenuManager from "./ContextMenuManager.svelte";
	import SelectionBox from "./SelectionBox.svelte";
	import ConfirmationDialog from "./ConfirmationDialog.svelte";
	import {
		watchStore,
		lifecycleLogger,
	} from "$lib/debug";
	import { vaultName } from "$lib/karta/VaultStore";
    import { type KartaEdge } from "$lib/types/types";

	onMount(() => {
		lifecycleLogger.log("Viewport mounted");

		// Setup reactive store logging
		watchStore(nodes, "NodeStore");
		watchStore(contexts, "ContextStore");
		watchStore(vaultName, "VaultStore");
	});

	let canvasContainer: HTMLElement;

	// Calculate inverse scale for constant screen size elements, to be passed to children
	$: inverseScale = 1 / viewTransform.current.scale;

	let canvas: HTMLElement;

	$: currentCtx = $contexts.get($currentContextId);

	// State for panning
	let isPanning = false;
	let panStartX = 0;
	let panStartY = 0;

	// State for last known cursor position
	let lastScreenX = 0;
	let lastScreenY = 0;

	// State for marquee selection
	let isMarqueeSelecting = false;
	let marqueeStartCoords: { canvasX: number; canvasY: number } | null = null;
	let marqueeEndCoords: { canvasX: number; canvasY: number } | null = null;
	let initialSelection: Set<string> | null = null;
	let marqueeRectElement: HTMLDivElement | null = null;

	// --- Constants for Scale Invariance ---
	const desiredScreenOutlineWidth = 1; // Target outline width in screen pixels

	// --- Helper Functions ---
	// Helper function to get image dimensions from an Object URL or Data URL
	function getImageDimensionsFromUrl(
		url: string,
	): Promise<{ width: number; height: number }> {
		return new Promise((resolve, reject) => {
			const img = new Image();
			img.onload = () => {
				resolve({ width: img.naturalWidth, height: img.naturalHeight });
			};
			img.onerror = (error) => {
				// Fallback to default dimensions if loading fails
				resolve({ width: 100, height: 100 }); // Or use registry defaults?
			};
			img.src = url;
		});
	}

	// --- Event Handlers ---

	// --- Connection Drag Handlers (Global Listeners) ---
	function handleConnectionPointerMove(event: PointerEvent) {
		if (!get(isConnecting) || !canvasContainer) return; // Check if connecting and container exists
		const rect = canvasContainer.getBoundingClientRect();
		const { x: canvasX, y: canvasY } = screenToCanvasCoordinates(
			event.clientX,
			event.clientY,
			rect,
		);
		updateTempLinePosition(canvasX, canvasY);
	}

	function handleConnectionPointerUp(event: PointerEvent) {
		if (!get(isConnecting) || event.button !== 0) return; // Only primary button release

		let targetNodeId: string | null = null;
		let currentElement: HTMLElement | null = event.target as HTMLElement;

		// Traverse up DOM to find a node element with data-id
		while (currentElement) {
			if (
				currentElement.dataset?.id &&
				currentElement.classList.contains("node-wrapper")
			) {
				targetNodeId = currentElement.dataset.id;
				break; // Found it
			}
			// Stop if we hit the canvas container or body
			if (
				currentElement === document.body ||
				currentElement === canvasContainer
			) {
				break;
			}
			currentElement = currentElement.parentElement;
		}

		// Check if the target node is a ghost node
		if (targetNodeId && !get(nodes).has(targetNodeId)) {
			cancelConnectionProcess(); // Cancel if target is ghost
		} else {
			finishConnectionProcess(targetNodeId); // Proceed if target is valid or null (background)
		}
		// Listeners are removed by the $effect cleanup
	}

	// --- Event Handlers ---
	function handleWheel(e: WheelEvent) {
		e.preventDefault();
		if (!canvasContainer) return;

		const rect = canvasContainer.getBoundingClientRect();
		const mouseX = e.clientX - rect.left;
		const mouseY = e.clientY - rect.top;
		const w = $viewportWidth;
		const h = $viewportHeight;

		const currentTransform = viewTransform.target;

		let newScale = currentTransform.scale;
		let newPosX = currentTransform.posX;
		let newPosY = currentTransform.posY;
		const panSensitivityFactor = 1.2; // Adjust this for faster/slower panning

		if (e.ctrlKey) {
			// Pinch-to-zoom (Ctrl key pressed)
			const zoomSensitivityFactor = 0.015;
			const zoomAmount = e.deltaY * -zoomSensitivityFactor;
			newScale = currentTransform.scale * (1 + zoomAmount);

			// Clamp the scale
			newScale = Math.max(0.1, Math.min(newScale, 5));

			// If we zoomed, recalculate position to keep the canvas point under the mouse
			if (newScale !== currentTransform.scale) {
				const canvasPointX = (mouseX - currentTransform.posX - w / 2) / currentTransform.scale;
				const canvasPointY = (mouseY - currentTransform.posY - h / 2) / currentTransform.scale;

				newPosX = mouseX - canvasPointX * newScale - w / 2;
				newPosY = mouseY - canvasPointY * newScale - h / 2;
			}
		} else {
			// Default to panning for both mouse wheel and touchpad
			newPosX = currentTransform.posX - e.deltaX * panSensitivityFactor;
			newPosY = currentTransform.posY - e.deltaY * panSensitivityFactor;
			// Keep scale the same when panning
			newScale = currentTransform.scale;
		}

		// Close menus if transform changes
		if (
			newScale !== currentTransform.scale ||
			newPosX !== currentTransform.posX ||
			newPosY !== currentTransform.posY
		) {
			closeContextMenu();
			closeCreateNodeMenu();
		}

		// Directly set the new transform without tweening for immediate response
		const newTransformWheel = {
			scale: newScale,
			posX: newPosX,
			posY: newPosY,
		};
		viewTransform.set(newTransformWheel, { duration: 0 });

		// Call tool's wheel handler if it exists
		get(currentTool)?.onWheel?.(e);
	}

	function handlePointerDown(e: PointerEvent) {
		// Middle mouse panning takes precedence
		if (e.button === 1) {
			e.preventDefault();
			isPanning = true;
			const currentTransform = viewTransform.target;
			panStartX = e.clientX - currentTransform.posX;
			panStartY = e.clientY - currentTransform.posY;
			// Ensure canvasContainer is bound before manipulating style/listeners
			if (canvasContainer) {
				canvasContainer.style.cursor = "grabbing";
				// Capture the pointer for this drag sequence
				canvasContainer.setPointerCapture(e.pointerId);
				// Add listeners directly to the element that captured the pointer
				canvasContainer.addEventListener(
					"pointermove",
					handleElementPointerMove,
				);
				canvasContainer.addEventListener(
					"pointerup",
					handleElementPointerUp,
				);
			} else {
			}
			return; // Don't delegate middle mouse
		}

		const targetElement = e.target as HTMLElement;
		const clickedOnNode = targetElement.closest(".node-wrapper");
		const clickedOnEdge = targetElement.closest(".edge-hit-area");
		const clickedOnBackground =
			targetElement === canvasContainer || targetElement === canvas;

		// --- Handle Edge Click & Reconnection ---
		if (clickedOnEdge && e.button === 0) {
			e.preventDefault();
			
			const edgeId = (clickedOnEdge as HTMLElement).dataset.edgeId;
			const endpoint = (clickedOnEdge as HTMLElement).dataset.endpoint as 'from' | 'to' | undefined;
			
			if (edgeId) {
				// Handle selection first (this should always happen)
				clearSelection(); // Clear node selection
				const currentEdgeSelection = get(selectedEdgeIds);
				const isSelected = currentEdgeSelection.has(edgeId);
				
				if (e.shiftKey || e.ctrlKey || e.metaKey) {
					toggleEdgeSelection(edgeId);
				} else {
					if (!isSelected || currentEdgeSelection.size > 1) {
						setSelectedEdges([edgeId]);
					} else {
						clearEdgeSelection();
					}
				}
				
				// Handle reconnection initiation if an endpoint was clicked
				if (endpoint) {
					startReconnectionProcess(edgeId, endpoint);
					
					// Add global listeners for the drag
					window.addEventListener('pointermove', handleReconnectionPointerMove);
					window.addEventListener('pointerup', handleReconnectionPointerUp, { once: true });
				}
			}
			
			return;
		}

		// --- Handle Background Click (and Marquee Start) ---
		if (clickedOnBackground && e.button === 0) {
			e.preventDefault();
			clearSelection();
			clearEdgeSelection();

			isMarqueeSelecting = true;

			// Calculate start coords
			const rect = canvasContainer.getBoundingClientRect();
			const { x: canvasX, y: canvasY } = screenToCanvasCoordinates(
				e.clientX,
				e.clientY,
				rect,
			);
			marqueeStartCoords = { canvasX, canvasY };
			marqueeEndCoords = { canvasX, canvasY }; // Initialize end coords
			initialSelection = new Set(get(selectedNodeIds)); // Store initial node selection (marquee only affects nodes)

			// Capture pointer on the container
			canvasContainer.setPointerCapture(e.pointerId);
			// Add move/up listeners specifically for marquee drag
			canvasContainer.addEventListener(
				"pointermove",
				handleMarqueePointerMove,
			);
			canvasContainer.addEventListener(
				"pointerup",
				handleMarqueePointerUp,
			);

			// Do NOT delegate to tool if starting marquee
			return;
		}

		// Need new handlers for marquee move/up, separate from general viewport move/up
		function handleMarqueePointerMove(e: PointerEvent) {
			if (!isMarqueeSelecting || !marqueeStartCoords || !canvasContainer)
				return;

			const rect = canvasContainer.getBoundingClientRect();
			const { x: currentCanvasX, y: currentCanvasY } =
				screenToCanvasCoordinates(e.clientX, e.clientY, rect);
			marqueeEndCoords = {
				canvasX: currentCanvasX,
				canvasY: currentCanvasY,
			};

			// --- Update Selection Logic ---
			updateSelectionFromMarquee(e.shiftKey, e.ctrlKey || e.metaKey);

			// --- Update Visual Marquee ---
			updateMarqueeVisual(); // We'll define this helper later
		}

		function handleMarqueePointerUp(e: PointerEvent) {
			if (!isMarqueeSelecting || !canvasContainer) return;

			// Check if it was a click (minimal movement) vs a drag
			const dx = marqueeEndCoords
				? Math.abs(
						marqueeEndCoords.canvasX - marqueeStartCoords!.canvasX,
					)
				: 0;
			const dy = marqueeEndCoords
				? Math.abs(
						marqueeEndCoords.canvasY - marqueeStartCoords!.canvasY,
					)
				: 0;
			const dragThreshold = 5 / viewTransform.current.scale;

			if (dx < dragThreshold && dy < dragThreshold) {
				clearSelection();
			}

			// Cleanup
			canvasContainer.releasePointerCapture(e.pointerId);
			canvasContainer.removeEventListener(
				"pointermove",
				handleMarqueePointerMove,
			);
			canvasContainer.removeEventListener(
				"pointerup",
				handleMarqueePointerUp,
			);

			isMarqueeSelecting = false;
			marqueeStartCoords = null;
			marqueeEndCoords = null;
			initialSelection = null;
			if (marqueeRectElement) {
				marqueeRectElement.remove();
				marqueeRectElement = null;
			}
		}

		// Helper function to calculate and update selection during marquee
		function updateSelectionFromMarquee(
			shiftKey: boolean,
			ctrlOrMetaKey: boolean,
		) {
			if (
				!isMarqueeSelecting ||
				!marqueeStartCoords ||
				!marqueeEndCoords ||
				!canvasContainer ||
				!initialSelection
			)
				return;

			const currentTransform = viewTransform.current;

			// Calculate marquee bounds in canvas coordinates
			const marqueeLeft = Math.min(
				marqueeStartCoords.canvasX,
				marqueeEndCoords.canvasX,
			);
			const marqueeRight = Math.max(
				marqueeStartCoords.canvasX,
				marqueeEndCoords.canvasX,
			);
			const marqueeTop = Math.min(
				marqueeStartCoords.canvasY,
				marqueeEndCoords.canvasY,
			);
			const marqueeBottom = Math.max(
				marqueeStartCoords.canvasY,
				marqueeEndCoords.canvasY,
			);

			const currentIntersectingIds = new Set<string>();
			const nodeElements = canvas?.querySelectorAll<HTMLElement>(".node-wrapper");

			nodeElements?.forEach((nodeEl) => {
				const nodeId = nodeEl.dataset.id;
				if (!nodeId) return;

				// Get node bounds (more accurate than viewNode state if possible)
				// This uses screen coords, needs conversion
				const nodeRect = nodeEl.getBoundingClientRect();
				const viewportRect = canvasContainer.getBoundingClientRect();

				// Convert node screen bounds to canvas bounds
				const nodeCanvasTopLeft = screenToCanvasCoordinates(
					nodeRect.left,
					nodeRect.top,
					viewportRect,
				);
				const nodeCanvasBottomRight = screenToCanvasCoordinates(
					nodeRect.right,
					nodeRect.bottom,
					viewportRect,
				);

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
				targetSelection = new Set([
					...initialSelection,
					...currentIntersectingIds,
				]);
			} else if (ctrlOrMetaKey) {
				targetSelection = new Set(
					[...initialSelection].filter(
						(id) => !currentIntersectingIds.has(id),
					),
				);
			} else {
				targetSelection = currentIntersectingIds;
			}

			// Update the main store directly
			selectedNodeIds.set(targetSelection); // Note: This might trigger many updates
		}

		// Creates or updates the visual marquee rectangle element
		function updateMarqueeVisual() {
			if (
				!isMarqueeSelecting ||
				!marqueeStartCoords ||
				!marqueeEndCoords ||
				!canvas // Check for canvas element now
			)
				return;

			// The marquee coordinates are already in canvas space.
			// We just need to calculate the top-left corner and dimensions.
			const left = Math.min(
				marqueeStartCoords.canvasX,
				marqueeEndCoords.canvasX,
			);
			const top = Math.min(
				marqueeStartCoords.canvasY,
				marqueeEndCoords.canvasY,
			);
			const width = Math.abs(
				marqueeStartCoords.canvasX - marqueeEndCoords.canvasX,
			);
			const height = Math.abs(
				marqueeStartCoords.canvasY - marqueeEndCoords.canvasY,
			);

			if (!marqueeRectElement) {
				// Create the element if it doesn't exist
				marqueeRectElement = document.createElement("div");
				marqueeRectElement.style.position = "absolute";
				// The border width needs to be scaled inversely to the viewport scale
				// to maintain a constant visual width.
				const currentScale = viewTransform.current.scale;
				const borderWidth = 1 / currentScale;
				marqueeRectElement.style.border = `${borderWidth}px dashed #cbd5e1`; // Tailwind slate-300
				marqueeRectElement.style.backgroundColor =
					"rgba(59, 130, 246, 0.1)"; // Tailwind blue-500 with opacity
				marqueeRectElement.style.pointerEvents = "none"; // Ignore pointer events
				marqueeRectElement.style.zIndex = "50"; // Ensure it's above nodes/edges
				marqueeRectElement.setAttribute("aria-hidden", "true");
				// Append to the transformed canvas, not the container
				canvas.appendChild(marqueeRectElement);
			}

			// Update position and size in canvas coordinates
			marqueeRectElement.style.left = `${left}px`;
			marqueeRectElement.style.top = `${top}px`;
			marqueeRectElement.style.width = `${width}px`;
			marqueeRectElement.style.height = `${height}px`;

			// Also update the border width on each frame, as the scale can change
			const currentScale = viewTransform.current.scale;
			const borderWidth = 1 / currentScale;
			marqueeRectElement.style.borderWidth = `${borderWidth}px`;
		}

		// If not middle mouse or marquee start, delegate to the active tool
		// Pass the event and the direct target element
		get(currentTool)?.onPointerDown?.(e, e.target as EventTarget | null);
	}

	// --- Reconnection Handlers ---
	function handleReconnectionPointerMove(event: PointerEvent) {
		if (!get(isReconnecting) || !canvasContainer) return;
		const rect = canvasContainer.getBoundingClientRect();
		const { x: canvasX, y: canvasY } = screenToCanvasCoordinates(
			event.clientX,
			event.clientY,
			rect,
		);
		updateTempLinePosition(canvasX, canvasY);
	}

	function handleReconnectionPointerUp(event: PointerEvent) {
		if (!get(isReconnecting)) return;

		let targetNodeId: string | null = null;
		let currentElement: HTMLElement | null = event.target as HTMLElement;

		while (currentElement) {
			if (currentElement.dataset?.id && currentElement.classList.contains('node-wrapper')) {
				targetNodeId = currentElement.dataset.id;
				break;
			}
			if (currentElement === document.body || currentElement === canvasContainer) {
				break;
			}
			currentElement = currentElement.parentElement;
		}

		finishReconnectionProcess(targetNodeId);

		// Cleanup global listeners
		window.removeEventListener('pointermove', handleReconnectionPointerMove);
	}

	// New handler for pointer move on the element during middle-mouse pan
	function handleElementPointerMove(e: PointerEvent) {
		// Check if we are still panning (redundant check if listeners are removed correctly, but safe)
		// Also check if the moving pointer is the one we captured
		// Add null check for canvasContainer
		if (
			isPanning &&
			canvasContainer &&
			canvasContainer.hasPointerCapture(e.pointerId)
		) {
			const newPosX = e.clientX - panStartX;
			const newPosY = e.clientY - panStartY;
			// Close menus if transform changes
			if (
				newPosX !== viewTransform.target.posX ||
				newPosY !== viewTransform.target.posY
			) {
				closeContextMenu();
				closeCreateNodeMenu();
			}
			const newTransformPan = {
				scale: viewTransform.target.scale,
				posX: newPosX,
				posY: newPosY,
			};
			viewTransform.set(newTransformPan, { duration: 0 });
		}
	}

	// New handler for pointer up on the element during middle-mouse pan
	function handleElementPointerUp(e: PointerEvent) {
		// Check if this is the up event for the pointer we captured and the middle button
		// Add null check for canvasContainer
		if (
			isPanning &&
			e.button === 1 &&
			canvasContainer &&
			canvasContainer.hasPointerCapture(e.pointerId)
		) {
			isPanning = false;
			// No need for inner null check now, already checked above
			canvasContainer.style.cursor = "default"; // Reset cursor
			// Remove listeners from the element
			canvasContainer.removeEventListener(
				"pointermove",
				handleElementPointerMove,
			);
			canvasContainer.removeEventListener(
				"pointerup",
				handleElementPointerUp,
			);
			// Release the pointer capture
			canvasContainer.releasePointerCapture(e.pointerId);
		}
	}

	// General pointer move on viewport (for non-panning moves, delegate to tool)
	function handleViewportPointerMove(e: PointerEvent) {
		// Changed to PointerEvent
		// Update last known cursor position
		lastScreenX = e.clientX;
		lastScreenY = e.clientY;

		// Delegate to the active tool
		get(currentTool)?.onPointerMove?.(e);
	}

	// General pointer up on viewport
	function handleViewportPointerUp(e: PointerEvent) {
		// Changed to PointerEvent
		// Delegate to the active tool
		get(currentTool)?.onPointerUp?.(e);
	}

	// Removed handleCanvasClick - click logic should be within tool's onPointerUp
	function handleKeyDown(e: KeyboardEvent) {
		// Check if focus is currently within an input, textarea, or contenteditable element
		const activeEl = document.activeElement;
		const isInputFocused =
			activeEl &&
			(activeEl.tagName === "INPUT" ||
				activeEl.tagName === "TEXTAREA" ||
				(activeEl instanceof HTMLElement &&
					activeEl.isContentEditable));

		if (e.key === "Tab") {
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
				} else {
				}

				// Calculate canvas coordinates
				const { x: canvasX, y: canvasY } = screenToCanvasCoordinates(
					screenX,
					screenY,
					rect,
				);

				try {
					const parentPath = findPhysicalParentPath(get(currentContextId));
					console.log(`[Viewport.handleKeyDown] findPhysicalParentPath result: ${parentPath}`);
					if (!parentPath.startsWith('vault')) {
						notifications.error("New nodes can only be created inside the Vault.", 4000);
						return;
					}
				} catch (error: any) {
					console.error(`[Viewport.handleKeyDown] findPhysicalParentPath error:`, JSON.stringify(error, null, 2));
					notifications.error(error.message, 5000);
					return;
				}

				// Open the menu
				openCreateNodeMenu(screenX, screenY, canvasX, canvasY);
				// DO NOT call e.preventDefault() here - it's handled globally
				return; // Prevent further handling if menu is opened
			} else {
			}
		} else if (e.key === "f" || e.key === "F") {
			// --- ADD THIS CHECK ---
			if (!isInputFocused) {
				// --- Existing logic moves inside ---
				// Handle both 'F' and 'Shift+F'
				if (e.shiftKey && !e.ctrlKey && !e.metaKey && !e.altKey) {
					// Shift+F: Frame Context
					e.preventDefault();
					frameContext();
				} else if (
					!e.shiftKey &&
					!e.ctrlKey &&
					!e.metaKey &&
					!e.altKey
				) {
					// F (no modifiers): Center Focal Node
					e.preventDefault();
					centerOnFocalNode();
				}
				// Ignore if other modifiers are pressed (e.g., Ctrl+F for browser search)
				// --- End of moved logic ---
			}
			// --- END OF ADDED CHECK ---
		}

		// Delegate keydown events to the active tool (unless handled above)
		if (!e.defaultPrevented) {
			get(currentTool)?.onKeyDown?.(e);
		}

		// Keep Escape key handling here for global cancel? Or move to tool?
		// Let's keep it global for now.
		if (e.key === "Escape") {
			cancelConnectionProcess();
			closeCreateNodeMenu(); // Also close create menu on Escape
			closeFilterMenu(); // Also close filter menu on Escape
			closeContextMenu(); // Also close context menu on Escape
		}
	}

	function handleKeyUp(e: KeyboardEvent) {
		// Delegate keyup events to the active tool
		get(currentTool)?.onKeyUp?.(e);
	}

	function handleContextMenu(e: MouseEvent) {
		// Prevent context menu if currently connecting
		if (get(isConnecting)) {
			e.preventDefault();
			cancelConnectionProcess(); // Cancel connection on right-click
			return;
		}

		e.preventDefault(); // Prevent default browser context menu

		const targetElement = e.target as HTMLElement;
		const clickedOnNode = targetElement.closest(".node-wrapper");
		const clickedOnEdge = targetElement.closest(".edge-hit-area"); // Check for edge hit area click

		let context: ContextMenuContextType;

		if (clickedOnNode) {
			const nodeId = (clickedOnNode as HTMLElement).dataset.id;
			context = { type: "node", id: nodeId };

			// --- New Logic: Update selection on right-click if node is not already selected ---
			const currentSelection = get(selectedNodeIds);
			if (nodeId && !currentSelection.has(nodeId)) {
				setSelectedNodes([nodeId]); // Clear existing and select only this node
			}
			// If the node is already selected, do nothing to the selection.
		} else if (clickedOnEdge) {
			// Corrected typo here
			const edgeId = (clickedOnEdge as HTMLElement).dataset.edgeId; // Corrected typo here
			context = { type: "edge", id: edgeId || "unknown" }; // Use unknown if ID not found yet
			
			// Add this selection logic for right-click on edges
			const currentEdgeSelection = get(selectedEdgeIds);
			if (edgeId && !currentEdgeSelection.has(edgeId)) {
				setSelectedEdges([edgeId]); // Clear existing and select only this edge
			}
			// If the edge is already selected, do nothing to the selection.
		} else {
			context = { type: "background" };
		}

		openContextMenu({ x: e.clientX, y: e.clientY }, context);
	}

	// Removed handleDoubleClick

	onMount(() => {
		// Focus the viewport container when the component mounts
		// This helps ensure keyboard events are captured correctly.
		canvasContainer?.focus();
	});

	// --- Lifecycle & Effects ---

	// Effect to manage global listeners for connection drag
	// Svelte 5: Use $effect rune
	// Svelte 4: Use $: reactive statement with a function call or onMount/onDestroy
	$: {
		// Reactive block for Svelte 4 effect simulation
		if (typeof window !== "undefined") {
			// Ensure runs only in browser
			if ($isConnecting) {
				window.addEventListener(
					"pointermove",
					handleConnectionPointerMove,
				);
				window.addEventListener("pointerup", handleConnectionPointerUp);
			} else {
				window.removeEventListener(
					"pointermove",
					handleConnectionPointerMove,
				);
				window.removeEventListener(
					"pointerup",
					handleConnectionPointerUp,
				);
			}
		}
	}

	// Ensure listeners are removed on component destroy
	onDestroy(() => {
		if (typeof window !== "undefined") {
			window.removeEventListener(
				"pointermove",
				handleConnectionPointerMove,
			);
			window.removeEventListener("pointerup", handleConnectionPointerUp);
			// Remove any other global listeners added here
		}
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

		const rect = canvasContainer.getBoundingClientRect();

		for (const item of e.dataTransfer.items) {
			if (item.kind === "file" && item.type.startsWith("image/")) {
				const file = item.getAsFile();
				if (file) {
					try {
						// Create Object URL (must be revoked later if creation fails)
						const objectUrl = URL.createObjectURL(file);
						const assetName = file.name || "Dropped Image";
						const { x: canvasX, y: canvasY } =
							screenToCanvasCoordinates(
								e.clientX,
								e.clientY,
								rect,
							);
						// Get dimensions from the object URL
						const dimensions =
							await getImageDimensionsFromUrl(objectUrl);
						// Call the new store action, passing the Blob, Object URL, and dimensions
						createImageNodeWithAsset(
							{ x: canvasX, y: canvasY },
							file,
							objectUrl,
							assetName,
							dimensions.width,
							dimensions.height,
						);
					} catch (error) {
						// console.error('[Viewport] Error reading dropped file:', error); // Keep error logs for now
					}
				}
			}
		}
		// Optional: Remove visual feedback added in handleDragOver
	}

	async function handlePaste(e: ClipboardEvent) {
		// Ignore paste events originating from inputs/textareas/contenteditables
		const target = e.target as HTMLElement;
		if (
			target.tagName === "INPUT" ||
			target.tagName === "TEXTAREA" ||
			target.isContentEditable
		) {
			return;
		}

		if (!e.clipboardData || !canvasContainer) return;

		e.preventDefault(); // Handle paste ourselves

		const rect = canvasContainer.getBoundingClientRect();
		let pasteCanvasX: number;
		let pasteCanvasY: number;

		// Determine paste position (last cursor or center)
		if (lastScreenX !== 0 || lastScreenY !== 0) {
			const coords = screenToCanvasCoordinates(
				lastScreenX,
				lastScreenY,
				rect,
			);
			pasteCanvasX = coords.x;
			pasteCanvasY = coords.y;
		} else {
			const centerX = rect.left + rect.width / 2;
			const centerY = rect.top + rect.height / 2;
			const coords = screenToCanvasCoordinates(centerX, centerY, rect);
			pasteCanvasX = coords.x;
			pasteCanvasY = coords.y;
		}

		for (const item of e.clipboardData.items) {
			if (item.kind === "file" && item.type.startsWith("image/")) {
				const file = item.getAsFile();
				if (file) {
					try {
						// Create Object URL (must be revoked later if creation fails)
						const objectUrl = URL.createObjectURL(file);
						const assetName = file.name || "Pasted Image";
						// Get dimensions from the object URL
						const dimensions =
							await getImageDimensionsFromUrl(objectUrl);
						// Call the new store action, passing the Blob, Object URL, and dimensions
						createImageNodeWithAsset(
							{ x: pasteCanvasX, y: pasteCanvasY },
							file,
							objectUrl,
							assetName,
							dimensions.width,
							dimensions.height,
						);
						return; // Handle first image found
					} catch (error) {
						// console.error('[Viewport] Error reading pasted file:', error); // Keep error logs for now
					}
				}
			} else if (item.kind === "string" && item.type === "text/plain") {
				item.getAsString((text) => {
					if (text && text.trim().length > 0) {
						createTextNodeFromPaste(
							{ x: pasteCanvasX, y: pasteCanvasY },
							text,
						);
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
	class="karta-viewport-container w-full h-screen overflow-hidden relative cursor-default bg-viewport-bg"
	bind:this={canvasContainer}
	bind:clientWidth={$viewportWidth}
	bind:clientHeight={$viewportHeight}
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
		style:transform="translate({viewTransform.current.posX +
			$viewportWidth / 2}px, {viewTransform.current.posY +
			$viewportHeight / 2}px) scale({viewTransform.current.scale})"
	>
		<!-- Edge Rendering Layer -->
		<EdgeLayer {inverseScale} />

		<!-- Node Rendering Layer - Iterate over ViewNodes in the current context -->
		{#if currentCtx}
			{#each [...currentCtx.viewNodes.values()] as viewNode (viewNode.id)}
				{@const dataNode = $nodes.get(viewNode.id)}
				<!-- Always render NodeWrapper; let it handle the ghost state if dataNode is missing -->
				<NodeWrapper {viewNode} {dataNode} />
			{/each}
		{/if}
		<!-- Selection Box -->
		{#if true}
			<!-- Wrap in valid block to fix {@const} placement -->
			{@const currentScaleValue = viewTransform.current.scale}
			{@const invScaleValue =
				currentScaleValue > 0 ? 1 / currentScaleValue : 1}
			{@const outlineWidthValue =
				currentScaleValue > 0
					? desiredScreenOutlineWidth / currentScaleValue
					: desiredScreenOutlineWidth}
			<SelectionBox
				inverseScale={invScaleValue}
				canvasOutlineWidth={outlineWidthValue}
			/>
		{/if}
	</div>

	<!-- Create Node Menu (conditionally rendered) -->
	{#if $isCreateNodeMenuOpen && $createNodeMenuPosition}
		<!-- Create Node Menu related elements -->
		{@const transform = viewTransform.current}
		<!-- Access tween value directly -->
		{@const markerScreenX =
			$createNodeMenuPosition.canvasX * transform.scale + transform.posX}
		{@const markerScreenY =
			$createNodeMenuPosition.canvasY * transform.scale + transform.posY}

		<!-- Position Marker (positioned using transformed canvas coords) -->
		<div
			class="absolute w-3 h-3 border-2 border-blue-400 rounded-full bg-blue-400 bg-opacity-30 pointer-events-none z-40"
			style:left="{markerScreenX - 6}px"
			style:top="{markerScreenY - 6}px"
			aria-hidden="true"
		></div>

		<!-- The Menu Component (positioned using screen coords) -->
		<CreateNodeMenu
			x={$createNodeMenuPosition.screenX + 10}
			y={$createNodeMenuPosition.screenY + 10}
		/>
	{/if}

	<!-- Create Node Menu elements remain outside the transformed canvas, positioned relative to viewport -->
</div>

<!-- Filter Menu (conditionally rendered) -->
{#if $isFilterMenuOpen && $filterMenuPosition}
	<FilterMenuDropdown
		x={$filterMenuPosition.screenX}
		y={$filterMenuPosition.screenY}
	/>
{/if}

<ContextMenuManager 
	{canvasContainer} 
	{screenToCanvasCoordinates} 
	{centerViewOnCanvasPoint} 
/>

<ConfirmationDialog />

