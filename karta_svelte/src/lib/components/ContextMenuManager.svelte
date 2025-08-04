<script lang="ts">
	import { get } from 'svelte/store';
	import {
		isContextMenuOpen,
		contextMenuPosition,
		contextMenuContext,
		openContextMenu,
		closeContextMenu,
		type ContextMenuContextType,
		requestNodeRename,
		openConfirmationDialog,
		openNodeSearch,
		openCreateNodeMenu,
	} from "$lib/karta/UIStateStore";
	import {
		selectedNodeIds,
		clearSelection,
		setSelectedNodes,
	} from "$lib/karta/SelectionStore";
	import {
		selectedEdgeIds,
		clearEdgeSelection,
		setSelectedEdges,
	} from "$lib/karta/EdgeSelectionStore";
	import {
		nodes,
		deleteDataNodePermanently,
		deleteSelectedNodesPermanently,
		findPhysicalParentPath,
	} from "$lib/karta/NodeStore";
	import { BundleExporter } from "$lib/export/BundleExporter";
	import { exportActions } from "$lib/karta/ExportStore";
	import { edges, deleteEdges } from "$lib/karta/EdgeStore";
	import {
		currentContextId,
		currentViewNodes,
		removeViewNodeFromContext,
		switchContext,
	} from "$lib/karta/ContextStore";
	import {
		centerOnFocalNode,
		frameContext,
	} from "$lib/karta/ViewportStore";
	import type { KartaEdge } from "$lib/types/types";
	import ContextMenu from "./ContextMenu.svelte";

	// Props for positioning calculations
	export let canvasContainer: HTMLElement | null = null;
	export let screenToCanvasCoordinates: (screenX: number, screenY: number, rect: DOMRect) => { x: number; y: number };
	export let centerViewOnCanvasPoint: (x: number, y: number) => void;

	let contextMenuElement: HTMLElement | null = null;

	// Handle click outside context menu
	function handleClickOutsideContextMenu(event: PointerEvent) {
		if (
			$isContextMenuOpen &&
			contextMenuElement &&
			!contextMenuElement.contains(event.target as Node)
		) {
			closeContextMenu();
		}
	}

	// Reactive statement for menu items - using Svelte 4 style for now, but properly reactive
	$: menuItems = (() => {
		let items: { label: string; action: () => void; disabled?: boolean }[] = [];
		
		const contextType = $contextMenuContext?.type;
		const targetNodeId = $contextMenuContext?.id;
		const currentNodesMap = $nodes;
		const currentViewNodesMap = $currentViewNodes;
		const screenPos = $contextMenuPosition;

		const targetDataNode = targetNodeId
			? currentNodesMap.get(targetNodeId)
			: null;
		const targetViewNode = targetNodeId
			? currentViewNodesMap.get(targetNodeId)
			: null;

		// Helper to get canvas coords from stored screen coords
		const getCanvasCoords = () => {
			if (!screenPos || !canvasContainer) return { x: 0, y: 0 };
			const rect = canvasContainer.getBoundingClientRect();
			return screenToCanvasCoordinates(screenPos.x, screenPos.y, rect);
		};

		if (contextType === "node" && targetNodeId) {
			const nodeState = targetViewNode?.state.current;
			const currentSelection = get(selectedNodeIds);
			const isMultipleSelected = currentSelection.size > 1;
			const selectedNodesList = Array.from(currentSelection);
			
			items = [
				{
					label: "Enter Context",
					action: () => {
						if (targetNodeId)
							switchContext({ type: "uuid", value: targetNodeId });
					},
					disabled:
						!targetNodeId || targetNodeId === $currentContextId,
				},
			];

			// Conditionally add "Remove from Context"
			const allEdges = get(edges);
			const focalNodeId = get(currentContextId);
			const isConnectedToFocal = [...allEdges.values()].some(edge =>
				(edge.source === targetNodeId && edge.target === focalNodeId) ||
				(edge.source === focalNodeId && edge.target === targetNodeId)
			);

			if (!isConnectedToFocal) {
				items.push({
					label: "Remove from Context",
					action: () => {
						const selected = get(selectedNodeIds);
						const currentCtxId = get(currentContextId);
						if (currentCtxId) {
							// Iterate through selected nodes and remove from context, skipping the focal node
							selected.forEach((nodeId) => {
								if (nodeId !== currentCtxId) {
									removeViewNodeFromContext(
										currentCtxId,
										nodeId,
									);
								}
							});
							clearSelection(); // Clear selection after removing
						}
					},
					// Disable if it's the focal node
					disabled:
						!targetNodeId || targetNodeId === $currentContextId,
				});
			}

			items.push(
				{
					label: "Center View",
					action: () => {
						if (nodeState) {
							centerViewOnCanvasPoint(
								nodeState.x + nodeState.width / 2,
								nodeState.y + nodeState.height / 2,
							);
						}
					},
					disabled: !nodeState,
				},
				{
					label: "Rename",
					action: () => {
						if (targetNodeId) requestNodeRename(targetNodeId);
					},
					disabled:
						isMultipleSelected ||
						!targetDataNode ||
						targetDataNode.attributes?.isSystemNode,
				},
				{
					label: (() => {
						const isDirectory = targetDataNode?.ntype === 'core/fs/dir';
						const isAlreadySelected = targetNodeId ? exportActions.isNodeSelected(targetNodeId) : false;
						
						if (isAlreadySelected) {
							return "Remove from Export Bundle";
						} else if (isDirectory) {
							return "Add Directory to Export Bundle";
						} else {
							return "Add to Export Bundle";
						}
					})(),
					action: () => {
						if (targetNodeId) {
							const isDirectory = targetDataNode?.ntype === 'core/fs/dir';
							const isAlreadySelected = exportActions.isNodeSelected(targetNodeId);
							
							if (isAlreadySelected) {
								exportActions.removeNode(targetNodeId);
							} else if (isDirectory) {
								exportActions.addDirectory(targetNodeId, true); // Recursive by default
							} else {
								exportActions.addNode(targetNodeId);
							}
						}
					},
					disabled: !targetNodeId,
				},
				{
					label: isMultipleSelected ? `Delete ${currentSelection.size} Items Permanently` : "Delete Permanently",
					action: () => {
						if (isMultipleSelected) {
							// Handle multiple selections
							const confirmationMessage = `Are you sure you want to permanently delete ${currentSelection.size} items? This action cannot be undone.`;
								
							openConfirmationDialog(
								confirmationMessage,
								async () => {
									// Call bulk deletion function
									await deleteSelectedNodesPermanently(selectedNodesList);
									clearSelection(); // Clear selection after deletion
								},
								selectedNodesList[0], // Use first selected node ID as placeholder
							);
						} else if (targetNodeId) {
							// Handle single selection
							const isDirectory = targetDataNode?.ntype === 'core/fs/dir';
							const confirmationMessage = isDirectory 
								? "Are you sure you want to permanently delete this folder? This will also delete all files and subfolders inside it. This action cannot be undone."
								: "Are you sure you want to permanently delete this item? This action cannot be undone.";
								
							openConfirmationDialog(
								confirmationMessage,
								async (idToDelete) => {
									await deleteDataNodePermanently(idToDelete);
								},
								targetNodeId,
							);
						}
					},
					disabled:
						(!targetNodeId && !isMultipleSelected) || 
						(targetNodeId === $currentContextId && !isMultipleSelected), // Disable if it's the focal node and not multiple selection
				}
			);
		} else if (contextType === "edge") {
			const selectedEdgesSet = get(selectedEdgeIds);
			const allKartaEdges = get(edges);

			const selectedEdges = Array.from(selectedEdgesSet)
				.map(id => allKartaEdges.get(id))
				.filter((edge): edge is KartaEdge => !!edge);

			const deletableEdges = selectedEdges.filter(edge => !edge.contains);

			items = [
				{
					label: `Delete ${deletableEdges.length} Edge(s)`,
					action: () => {
						if (deletableEdges.length === 0) return;

						const payloads = deletableEdges.map(edge => ({ source: edge.source, target: edge.target }));

						if (payloads.length > 0) {
							deleteEdges(payloads);
							clearEdgeSelection();
						}
						closeContextMenu();
					},
					disabled: deletableEdges.length === 0,
				},
			];
		} else if (contextType === "background") {
			let canCreateNodes = false;
			try {
				const parentPath = findPhysicalParentPath(get(currentContextId));
				console.log(`[ContextMenuManager] findPhysicalParentPath result: ${parentPath}`);
				if (parentPath.startsWith('vault')) {
					canCreateNodes = true;
				}
			} catch (error: any) {
				console.error(`[ContextMenuManager] findPhysicalParentPath error:`, JSON.stringify(error, null, 2));
				// Do nothing, canCreateNodes remains false
			}

			items = [
				{
					label: "Center Focal Node",
					action: () => centerOnFocalNode(),
					disabled: !$currentContextId,
				},
				{
					label: "Frame Context",
					action: () => frameContext(),
					disabled: !$currentContextId,
				},
				{
					label: canCreateNodes ? "Create Node Here" : "Can't create node here",
					action: () => {
						const { x: canvasX, y: canvasY } = getCanvasCoords();
						openCreateNodeMenu(
							screenPos!.x,
							screenPos!.y,
							canvasX,
							canvasY,
						);
					},
					disabled: !screenPos || !canCreateNodes,
				},
				{
					label: canCreateNodes ? "Search nodes..." : "Can't search nodes here",
					action: () => {
						const { x: canvasX, y: canvasY } = getCanvasCoords();
						openNodeSearch(
							screenPos!.x,
							screenPos!.y,
							canvasX,
							canvasY,
						);
					},
					disabled: !screenPos || !canCreateNodes,
				},
			];
		}
		return items;
	})();
</script>

<svelte:window on:pointerdown={handleClickOutsideContextMenu} />

{#if $isContextMenuOpen && $contextMenuPosition}
	<div bind:this={contextMenuElement}>
		<ContextMenu position={$contextMenuPosition} items={menuItems} />
	</div>
{/if}
