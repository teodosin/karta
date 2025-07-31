import { writable, get } from 'svelte/store';
import { setSelectedNodes } from './SelectionStore';
import { localAdapter } from '../util/LocalAdapter';
import { ServerAdapter } from '../util/ServerAdapter';
import type { DataNode, NodeId, AssetData, TweenableNodeState, ViewNode, Context, KartaEdge } from '../types/types';
import { v4 as uuidv4 } from 'uuid';
import { getDefaultAttributesForType, getDefaultViewNodeStateForType } from '$lib/node_types/registry';
import { Tween } from 'svelte/motion';
import { currentContextId, contexts, removeViewNodeFromContext, existingContextsMap, activeAdapter } from './ContextStore';
import { cubicInOut } from 'svelte/easing';
import { viewTransform, centerViewOnCanvasPoint } from './ViewportStore';

// Animation constants (matching ContextStore)
const NODE_TWEEN_DURATION = 1000;
export const nodes = writable<Map<NodeId, DataNode>>(new Map());

// Use a generic persistence service, which can be LocalAdapter or ServerAdapter
const persistenceService = new ServerAdapter(); // Or dynamically switch based on config

async function _ensureDataNodeExists(nodeId: NodeId): Promise<DataNode | null> {
    if (!persistenceService) {
        console.error(`[_ensureDataNodeExists] Persistence service not available while checking for ${nodeId}`);
        return null;
    }
    try {
        let dataNode = await persistenceService.getNode(nodeId);

        if (!dataNode) {
            // Create default DataNode if not found
            const now = Date.now();
            // Determine default properties based on whether it's the root node
            const isRoot = nodeId === ROOT_NODE_ID;
            const defaultName = isRoot ? 'root' : `node-${nodeId.substring(0, 8)}`;
            const defaultPath = isRoot ? '/root' : `/${defaultName}`;
            const defaultNtype = isRoot ? 'core/root' : 'core/generic';
            if (isRoot) {
            }

            dataNode = {
                id: nodeId,
                ntype: defaultNtype,
                createdAt: now,
                modifiedAt: now,
                path: defaultPath,
                attributes: { name: defaultName, ...(isRoot && { isSystemNode: true, view_isNameVisible: false }) },
            };
            // This initial save might need to be adapted for ServerAdapter if it requires a parent path
            // For now, we assume it can handle a simple save.
            await persistenceService.saveNode(dataNode);
        }
        return dataNode;
    } catch (error) {
        console.error(`[_ensureDataNodeExists] Error ensuring DataNode ${nodeId} exists:`, error);
        return null;
    }
}

export async function createNodeAtPosition(
    canvasX: number,
    canvasY: number,
    ntype: string = 'core/text',
    attributes: Record<string, any> = {},
    initialWidth?: number,
    initialHeight?: number
): Promise<NodeId | null> {

    const newNodeId: NodeId = uuidv4();
    const now = Date.now();
    const baseName = attributes.name || ntype;

    // 1. Create DataNode
    // The client no longer determines the finalName or path. It sends the desired name,
    // and the server returns the authoritative state.
    const newNodeData: DataNode = {
        id: newNodeId, ntype: ntype, createdAt: now, modifiedAt: now,
        path: `/${baseName}`, // Path will be corrected by the server response.
        attributes: {
            ...getDefaultAttributesForType(ntype),
            ...attributes,
            name: baseName
        },
    };

    // 2. Get default view state based on ntype and create initial state for the ViewNode's tween
    const defaultViewState = getDefaultViewNodeStateForType(ntype); // Gets { width, height, scale, rotation }
    const initialState: TweenableNodeState = {
        x: canvasX,
        y: canvasY,
        width: initialWidth ?? newNodeData.attributes.view_width, // Use provided width or default from DataNode attributes
        height: initialHeight ?? newNodeData.attributes.view_height, // Use provided height or default from DataNode attributes
        scale: defaultViewState.scale, // Keep default scale (scale is not managed as a view_ attribute yet)
        rotation: defaultViewState.rotation // Keep default rotation (rotation is not managed as a view_ attribute yet)
    };

    // 3. Create the new ViewNode containing the Tween
    const newViewNode: ViewNode = {
        id: newNodeId,
        state: new Tween(initialState, { duration: 0 }), // Initialize instantly, no animation
        status: 'modified'
    };

    // 4. Update stores
    const contextId = get(currentContextId);
    // We will add the node to the store *after* it has been successfully created on the server.
    // This prevents race conditions where updates are sent for a node that doesn't exist yet.

    // 5. Persist changes
    if (persistenceService) {
        try {
            // For ServerAdapter, we need the parent path.
            const parentPath = findPhysicalParentPath(contextId);

            // Debug: Creating node at position
            const persistedNode = await (persistenceService as ServerAdapter).createNode(newNodeData, parentPath);

            if (!persistedNode) {
                console.error("[NodeStore.createNodeAtPosition] Node creation failed on the server.");
                throw new Error("Node creation failed on the server.");
            }
            // Debug: Server returned persisted node

            // NOW, update the stores with the definitive node data from the server
            nodes.update(n => n.set(persistedNode.id, persistedNode)); // Add the persisted DataNode

            // CRITICAL FIX: The ViewNode was created with a temporary client-side ID.
            // We need to update the contexts store to use the server-authoritative ID.
            contexts.update((ctxMap: Map<NodeId, Context>) => {
                // Debug: Context map before update
                let currentCtx = ctxMap.get(contextId);
                if (!currentCtx) {
                    // Create context if it doesn't exist
                    currentCtx = { id: contextId, viewNodes: new Map() };
                    ctxMap.set(contextId, currentCtx);
                }
                // Remove the old ViewNode with the temporary ID and add it back with the correct ID.
                currentCtx.viewNodes.delete(newNodeId);
                newViewNode.id = persistedNode.id; // Update the ViewNode's ID
                currentCtx.viewNodes.set(persistedNode.id, newViewNode);
                // Debug: Context map after update
                return ctxMap;
            });

            // Optimistically create the edge to the parent
            const allNodes = get(nodes);
            const parentNode = Array.from(allNodes.values()).find(n => n.path === parentPath);
            if (parentNode) {
                const newEdge: KartaEdge = {
                    id: uuidv4(),
                    source: parentNode.id,
                    target: persistedNode.id,
                    attributes: {},
                    contains: true,
                };
                edges.update(e => e.set(newEdge.id, newEdge));
            }

            return persistedNode.id; // Return ID on successful save
        } catch (error) {
            console.error("Error saving node or context after creation:", error);
            return null;
        }
    } else {
        // Persistence service not available - still add to stores for non-persistent use
        return newNodeId;
        // Or return null if persistence is strictly required? Let's return ID for now.
    }
}

export function findPhysicalParentPath(contextId: NodeId): string {
    const allNodes = get(nodes);
    const startNode = allNodes.get(contextId);

    if (!startNode) {
        throw new Error(`[findPhysicalParentPath] Could not find node data for starting context ID: ${contextId}`);
    }

    // If the starting context node itself is a directory, it's the parent.
    if (startNode.ntype === 'core/fs/dir') {
        return startNode.path;
    }

    // If the starting node is not a directory, its immediate parent must be the physical parent.
    const parentPathStr = startNode.path.substring(0, startNode.path.lastIndexOf('/')) || '/';
    const parentNode = Array.from(allNodes.values()).find(n => n.path === parentPathStr);

    if (!parentNode) {
        throw new Error(`[findPhysicalParentPath] Could not find parent node with path: '${parentPathStr}'`);
    }

    // As a final sanity check, ensure the found parent is actually a directory.
    if (parentNode.ntype !== 'core/fs/dir') {
        throw new Error(`[findPhysicalParentPath] Invalid State: Parent node '${parentNode.path}' is not a directory.`);
    }

    console.log(`[findPhysicalParentPath] Found physical parent: '${parentNode.path}'`);
    return parentNode.path;
}

export async function createImageNodeFromDataUrl(position: { x: number, y: number }, dataUrl: string, width?: number, height?: number) {
    try {
        // Create the basic node structure first
        // Pass optional width and height to createNodeAtPosition
        const newNodeId = await createNodeAtPosition(position.x, position.y, 'core/image', {}, width, height);
        if (!newNodeId) {
            console.error("[KartaStore] Failed to create base node for image paste.");
            return;
        }

        // Update the attributes with the image source
        // TODO: Consider adding warnings or size limits for very large images.
        await updateNodeAttributes(newNodeId, { src: dataUrl });

    } catch (error) {
        console.error("[KartaStore] Error creating image node from Data URL:", error);
    }
}

export async function createTextNodeFromPaste(position: { x: number, y: number }, text: string) {
    try {
        // Create the basic node structure first
        const newNodeId = await createNodeAtPosition(position.x, position.y, 'core/text');
        if (!newNodeId) {
            console.error("[KartaStore] Failed to create base node for text paste.");
            return;
        }

        // Update the attributes with the pasted text
        await updateNodeAttributes(newNodeId, { text: text });

    } catch (error) {
        console.error("[KartaStore] Error creating text node from paste:", error);
    }
}

export async function createImageNodeWithAsset(
    position: { x: number, y: number },
    imageBlob: Blob,
    objectUrl: string, // Pre-generated Object URL
    assetName: string,
    initialWidth?: number,
    initialHeight?: number
): Promise<NodeId | null> {
    if (!persistenceService) {
        console.error("[createImageNodeWithAsset] Persistence service not available.");
        URL.revokeObjectURL(objectUrl); // Revoke if we can't save
        return null;
    }

    let newNodeId: NodeId | null = null;
    try {
        // 1. Create the base 'image' node with alt text, but no src initially
        // Use the existing createNodeAtPosition function
        newNodeId = await createNodeAtPosition(
            position.x,
            position.y,
            'core/image',
            { alt: assetName, name: assetName }, // Set alt and name attributes initially
            initialWidth,
            initialHeight
        );

        if (!newNodeId) {
            throw new Error("Failed to create base node structure.");
        }

        // 2. Prepare asset data and save it using the newNodeId as assetId
        const assetData: AssetData = { blob: imageBlob, mimeType: imageBlob.type, name: assetName };
        await persistenceService.saveAsset(newNodeId, assetData);

        // 3. Update the node attributes with the Object URL and assetId
        // Use the existing updateNodeAttributes function
        await updateNodeAttributes(newNodeId, { src: objectUrl, assetId: newNodeId });

        return newNodeId;

    } catch (error) {
        console.error("[createImageNodeWithAsset] Error:", error);
        // Cleanup logic:
        if (newNodeId) {
            // If the base node was created, attempt to delete it from DB.
            // Use the batch deleteNodes method (deleteNode was removed)
            await persistenceService.deleteNodes([newNodeId]);
            // Also remove from stores manually
            nodes.update(n => { n.delete(newNodeId!); return n; });
            contexts.update((ctxMap: Map<NodeId, Context>) => { // Explicitly type ctxMap
                ctxMap.forEach(ctx => ctx.viewNodes.delete(newNodeId!));
                return ctxMap;
            });
        }
        // Regardless of whether the node was created, revoke the initially passed objectUrl
        // as it might not have been tracked/revoked by the adapter if saveAsset failed.
        URL.revokeObjectURL(objectUrl);

        return null; // Indicate failure
    }
}


export async function updateNodeAttributes(nodeId: NodeId, newAttributes: Record<string, any>) {
    const currentNodes = get(nodes);
    const dataNode = currentNodes.get(nodeId);

    if (!dataNode) {
        // DataNode not found in store
        return;
    }

    // Prevent renaming system nodes
    if (dataNode.attributes?.isSystemNode) {
        // Cannot modify system node attributes
        return;
    }

    const oldName = dataNode.attributes?.name;
    const newName = newAttributes?.name;
    let attributesToSave = { ...dataNode.attributes, ...newAttributes }; // Merge old and new
    // The client no longer handles name collision logic on update.
    // It sends the desired attributes, and the server is responsible for any validation.
    if (newAttributes.name !== undefined && !newAttributes.name.trim()) {
        // Prevent empty names
        return;
    }

    // Create updated node data
    const updatedNodeData: DataNode = {
        id: dataNode.id,
        ntype: dataNode.ntype,
        createdAt: dataNode.createdAt,
        path: dataNode.path, // Keep the original path - server will return the correct updated path
        attributes: attributesToSave, // Use the final attributes map (name might have been incremented)
        modifiedAt: Date.now(),
    };

    // Update the store
    // Update the store only if changes were actually made
    // Compare the final attributesToSave with the original dataNode attributes
    if (JSON.stringify(attributesToSave) !== JSON.stringify(dataNode.attributes)) {
        nodes.update(n => n.set(nodeId, updatedNodeData));

        // Persist changes
        if (persistenceService) {
            try {
                let persistedNode: DataNode | undefined;
                
                // Check if this is a rename operation and if the node has a path but might be unindexed
                const isRenameOperation = newAttributes.name !== undefined && newAttributes.name !== oldName;
                
                if (isRenameOperation && dataNode.path && (persistenceService as any).renameNode) {
                    // Try the path-based rename endpoint first for rename operations
                    console.log(`[updateNodeAttributes] Using path-based rename for node at path "${dataNode.path}"`);
                    try {
                        persistedNode = await (persistenceService as any).renameNode(dataNode.path, newAttributes.name);
                    } catch (renameError) {
                        console.log(`[updateNodeAttributes] Path-based rename failed, falling back to updateNode:`, renameError);
                        // Fall back to the regular updateNode method
                        persistedNode = await (persistenceService as ServerAdapter).updateNode(updatedNodeData);
                    }
                } else {
                    // Use the regular updateNode method for non-rename operations
                    persistedNode = await (persistenceService as ServerAdapter).updateNode(updatedNodeData);
                }
                
                if (persistedNode) {
                    // Re-update the store with the authoritative data from the server
                    nodes.update(n => n.set(nodeId, persistedNode));
                }
            } catch (error) {
                console.error(`[updateNodeAttributes] Error saving node ${nodeId} after attribute update:`, error);
                // Optionally revert store update on save failure?
                // nodes.update(n => n.set(nodeId, dataNode)); // Example revert
            }
        } else {
            console.warn("[updateNodeAttributes] Persistence service not initialized, persistence disabled.");
        }
    } else {
    }
}
export async function updateNodeSearchableFlag(nodeId: NodeId, isSearchable: boolean): Promise<void> {
    const currentNodes = get(nodes);
    const dataNode = currentNodes.get(nodeId);

    if (!dataNode) {
        // DataNode not found in store
        return;
    }

    // Check if the value is actually changing
    // Treat undefined as true for comparison purposes
    const currentIsSearchable = dataNode.isSearchable ?? true;
    if (currentIsSearchable === isSearchable) {
        return; // No change needed
    }

    // Create updated node data immutably
    const updatedNodeData: DataNode = {
        ...dataNode,
        isSearchable: isSearchable,
        modifiedAt: Date.now(), // Update modified time
    };

    // Update the store
    nodes.update(n => n.set(nodeId, updatedNodeData));

    // Persist changes
    if (persistenceService) {
        try {
            await (persistenceService as ServerAdapter).updateNode(updatedNodeData);
        } catch (error) {
            console.error(`[updateNodeSearchableFlag] Error saving node ${nodeId} after searchable flag update:`, error);
            // Optionally revert store update on save failure?
            // nodes.update(n => n.set(nodeId, dataNode)); // Example revert
        }
    } else {
        console.warn("[updateNodeSearchableFlag] Persistence service not initialized, persistence disabled.");
    }
}


export async function fetchAvailableContextDetails(): Promise<{ id: NodeId, name: string, path: string }[]> {
    if (!persistenceService) {
        console.error("[fetchAvailableContextDetails] Persistence service not available.");
        return [];
    }
    try {
        const contextIds = await persistenceService.getAllContextIds();
        if (contextIds.length === 0) {
            return [];
        }

        const dataNodesMap = await persistenceService.getDataNodesByIds(contextIds);
        // Add type assertion for iterator
        const contextDetails = Array.from(dataNodesMap.values() as IterableIterator<DataNode>)
            .map((node: DataNode) => ({ // Explicitly type node
                id: node.id,
                name: node.attributes?.name ?? `Node ${node.id.substring(0, 8)}`, // Fallback name
                path: node.path ?? `/${node.attributes?.name ?? node.id.substring(0, 8)}` // Fallback path
            }))
            .sort((a, b) => a.path.localeCompare(b.path)); // Sort alphabetically by path

        return contextDetails;

    } catch (error) {
        console.error("[fetchAvailableContextDetails] Error fetching context details:", error);
        return [];
    }
}

// Import necessary stores and types for deletion
import { edges, deleteEdges } from './EdgeStore';
import { ROOT_NODE_ID } from '$lib/constants';

export async function deleteDataNodePermanently(nodeId: NodeId): Promise<void> {
    if (!persistenceService) {
        console.error("[deleteDataNodePermanently] Persistence service not available.");
        return;
    }

    try {
        console.log(`[deleteDataNodePermanently] ===== STARTING DELETION PROCESS FOR NODE ${nodeId} =====`);
        
        // 1. Get the node data to determine the best deletion approach
        const nodeData = get(nodes).get(nodeId);
        let nodeHandle = nodeId; // Default to UUID
        
        console.log(`[deleteDataNodePermanently] Node data:`, nodeData);
        console.log(`[deleteDataNodePermanently] Node type: ${nodeData?.ntype}, Path: ${nodeData?.path}`);
        
        // 2. Use path instead of UUID when available (works for both indexed and unindexed files)
        if (nodeData?.path) {
            nodeHandle = nodeData.path;
            console.log(`[deleteDataNodePermanently] Using path for deletion: ${nodeHandle}`);
        } else {
            console.log(`[deleteDataNodePermanently] Using UUID for deletion: ${nodeHandle}`);
        }
        
        // Get current store state before deletion for comparison
        const allNodesBefore = get(nodes);
        const allEdgesBefore = get(edges);
        console.log(`[deleteDataNodePermanently] Store state before deletion: ${allNodesBefore.size} nodes, ${allEdgesBefore.size} edges`);
        console.log(`[deleteDataNodePermanently] All node IDs before deletion:`, Array.from(allNodesBefore.keys()));
        
        // 3. Try deletion with the chosen handle (path preferred, UUID fallback)
        console.log(`[deleteDataNodePermanently] Calling server deleteNodes([${nodeHandle}])`);
        const deleteResponse = await persistenceService.deleteNodes([nodeHandle]);
        console.log(`[deleteDataNodePermanently] Server response:`, deleteResponse);
        
        // 3. Check for server failures
        if (deleteResponse.failed_deletions.length > 0) {
            const failure = deleteResponse.failed_deletions[0];
            console.error(`[deleteDataNodePermanently] Server failed to delete node ${nodeId}: ${failure.error}`);
            throw new Error(`Failed to delete node: ${failure.error}`);
        }

        // 4. Process successfully deleted nodes from server response
        console.log(`[deleteDataNodePermanently] Processing server response: ${deleteResponse.deleted_nodes.length} deleted nodes`);
        const deletedNodeIds = new Set<NodeId>();
        for (const deletedInfo of deleteResponse.deleted_nodes) {
            console.log(`[deleteDataNodePermanently] Server deleted node ${deletedInfo.node_id}, with ${deletedInfo.descendants_deleted.length} descendants`);
            console.log(`[deleteDataNodePermanently] Descendants deleted:`, deletedInfo.descendants_deleted);
            
            deletedNodeIds.add(deletedInfo.node_id);
            // Also add all descendants that were deleted
            for (const descendantId of deletedInfo.descendants_deleted) {
                deletedNodeIds.add(descendantId);
            }
        }
        
        console.log(`[deleteDataNodePermanently] Total nodes to delete from client stores: ${deletedNodeIds.size}`);
        console.log(`[deleteDataNodePermanently] All deleted node IDs:`, Array.from(deletedNodeIds));

        // 5. Remove all deleted nodes from the nodes store
        // ViewNodes in other contexts will automatically become ghost nodes
        console.log(`[deleteDataNodePermanently] Removing ${deletedNodeIds.size} nodes from nodes store`);
        nodes.update(nodeMap => {
            for (const deletedId of deletedNodeIds) {
                const deletedNode = nodeMap.get(deletedId);
                if (deletedNode) {
                    console.log(`[deleteDataNodePermanently] Removing node ${deletedId} (${deletedNode.ntype}) from store`);
                } else {
                    console.log(`[deleteDataNodePermanently] Node ${deletedId} not found in store (already removed?)`);
                }
                nodeMap.delete(deletedId);
            }
            return nodeMap;
        });

        // 6. Remove edges connected to deleted nodes from edges store
        console.log(`[deleteDataNodePermanently] Checking for edges to remove...`);
        const allEdges = get(edges);
        console.log(`[deleteDataNodePermanently] Total edges in store: ${allEdges.size}`);
        const edgesToDelete = [...allEdges.values()].filter(edge => 
            deletedNodeIds.has(edge.source) || deletedNodeIds.has(edge.target)
        );
        
        console.log(`[deleteDataNodePermanently] Found ${edgesToDelete.length} edges to delete`);
        for (const edge of edgesToDelete) {
            console.log(`[deleteDataNodePermanently] Will delete edge ${edge.id}: ${edge.source} -> ${edge.target} (contains: ${edge.contains})`);
        }
        
        if (edgesToDelete.length > 0) {
            edges.update(edgeMap => {
                for (const edge of edgesToDelete) {
                    edgeMap.delete(edge.id);
                }
                return edgeMap;
            });
            
            console.log(`[deleteDataNodePermanently] Removed ${edgesToDelete.length} edges from store for deleted nodes`);
        }

        // 7. Remove ViewNode from current context only
        const currentCtxId = get(currentContextId);
        console.log(`[deleteDataNodePermanently] Current context ID: ${currentCtxId}`);
        if (currentCtxId) {
            // Use the first deleted node ID (which should be our target node or its new UUID)
            const targetNodeId = deleteResponse.deleted_nodes[0]?.node_id || nodeId;
            console.log(`[deleteDataNodePermanently] Removing ViewNode ${targetNodeId} from current context ${currentCtxId}`);
            await removeViewNodeFromContext(currentCtxId, targetNodeId);
        }

        // 8. Delete contexts for deleted nodes and update the existingContextsMap
        console.log(`[deleteDataNodePermanently] Cleaning up contexts for ${deletedNodeIds.size} deleted nodes`);
        for (const deletedId of deletedNodeIds) {
            try {
                console.log(`[deleteDataNodePermanently] Attempting to delete context for node ${deletedId}`);
                await persistenceService.deleteContext(deletedId);
                existingContextsMap.update(map => {
                    map.delete(deletedId);
                    return map;
                });
                console.log(`[deleteDataNodePermanently] Successfully deleted context for node ${deletedId}`);
            } catch (contextDeleteError) {
                // Log but continue - context deletion is not critical
                console.warn(`[deleteDataNodePermanently] Error deleting context for node ${deletedId}:`, contextDeleteError);
            }
        }

        // 9. Log success with details from server response
        const mainDeletion = deleteResponse.deleted_nodes.find((d: any) => d.node_id === nodeId || deletedNodeIds.has(d.node_id));
        if (mainDeletion) {
            const totalDeleted = deletedNodeIds.size;
            const fileType = mainDeletion.was_physical ? "physical file/directory" : "virtual node";
            const identifier = nodeData?.path || nodeId; // Use path if available, otherwise UUID
            
            console.log(`[deleteDataNodePermanently] Successfully deleted ${fileType} at ${identifier}` + 
                       (totalDeleted > 1 ? ` and ${totalDeleted - 1} related nodes` : ''));
            
            if (deleteResponse.warnings.length > 0) {
                console.warn(`[deleteDataNodePermanently] Warnings: ${deleteResponse.warnings.join(', ')}`);
            }
        }
        
        // Get final store state for comparison
        const allNodesAfter = get(nodes);
        const allEdgesAfter = get(edges);
        console.log(`[deleteDataNodePermanently] Store state after deletion: ${allNodesAfter.size} nodes, ${allEdgesAfter.size} edges`);
        console.log(`[deleteDataNodePermanently] All node IDs after deletion:`, Array.from(allNodesAfter.keys()));
        console.log(`[deleteDataNodePermanently] ===== DELETION PROCESS COMPLETED =====`);

    } catch (error) {
        console.error(`[deleteDataNodePermanently] ERROR: Exception during deletion of node ${nodeId}:`, error);
        throw error; // Re-throw for caller to handle
    }
}


export async function ensureNodesExist(nodeIds: NodeId[]): Promise<void> {
	const currentNodes = get(nodes);
	const missingNodeIds = nodeIds.filter((id) => !currentNodes.has(id));

	if (missingNodeIds.length === 0) {
		return;
	}

	if (persistenceService) {
		try {
			const fetchedNodesPromises = missingNodeIds.map(id => (persistenceService as ServerAdapter).getNode(id));
			const fetchedNodes = (await Promise.all(fetchedNodesPromises)).filter(n => n !== undefined) as DataNode[];
			
			nodes.update((n) => {
				for (const node of fetchedNodes) {
					n.set(node.id, node);
				}
				return n;
			});
		} catch (error) {
			// Error fetching missing nodes
		}
	} else {
		// Persistence service not initialized
	}
}

export { _ensureDataNodeExists };

// --- Generic ViewNode Attribute Update ---
export async function updateViewNodeAttribute(viewNodeId: string, attributeKey: string, attributeValue: any): Promise<void> {
    const currentCtxId = get(currentContextId);
    if (!currentCtxId) {
        console.error("[updateViewNodeAttribute] Cannot update attribute: No current context ID");
        return;
    }

    const currentContexts = get(contexts);
    const currentContext = currentContexts.get(currentCtxId);
    if (!currentContext) {
        console.error(`[updateViewNodeAttribute] Cannot update attribute: Context ${currentCtxId} not found`);
        return;
    }

    const viewNode = currentContext.viewNodes.get(viewNodeId);
    if (!viewNode) {
        console.error(`[updateViewNodeAttribute] Cannot update attribute: ViewNode ${viewNodeId} not found in context ${currentCtxId}`);
        return;
    }

    const dataNodeId = viewNode.id; // ViewNode ID is same as DataNode ID
    const allNodes = get(nodes);
    const dataNode = allNodes.get(dataNodeId);

    if (!dataNode) {
        console.error(`[updateViewNodeAttribute] Cannot update attribute: DataNode ${dataNodeId} not found`);
        return;
    }

    // --- Type Check ---
    let isValidAttribute = true; // Assume valid by default
    if (attributeKey.startsWith('type_')) {
        isValidAttribute = false;
        console.error(`[updateViewNodeAttribute] Attribute key "${attributeKey}" starts with 'type_' and cannot be set on a ViewNode.`);
    }
    // Add more specific validation for known view_* and viewtype_* attributes if needed,
    // or ensure the component handles unknown attributes gracefully.
    // For now, allow any unprefixed, view_*, or viewtype_* attribute.

    if (!isValidAttribute) {
        return;
    }

    // --- Determine if updates are needed (compare with original values) ---
    const needsViewNodeUpdate = viewNode.attributes?.[attributeKey] !== attributeValue;
    let needsDataNodeUpdate = false;

    // Determine if DataNode also needs update (only for view_ and viewtype_ attributes)
    if (attributeKey.startsWith('view_') || attributeKey.startsWith('viewtype_')) {
        needsDataNodeUpdate = dataNode.attributes?.[attributeKey] !== attributeValue;
    }

    // --- Update ViewNode (Context Specific) ---
    if (needsViewNodeUpdate) {
        contexts.update(ctxMap => {
            const originalContext = ctxMap.get(currentCtxId);
            if (!originalContext) return ctxMap;
            const originalViewNode = originalContext.viewNodes.get(viewNodeId);
            if (!originalViewNode) return ctxMap;

            const newViewAttributes = { ...(originalViewNode.attributes ?? {}), [attributeKey]: attributeValue };
            const newViewNode = { ...originalViewNode, attributes: newViewAttributes };
            const newViewNodes = new Map(originalContext.viewNodes).set(viewNodeId, newViewNode);
            const newContext = { ...originalContext, viewNodes: newViewNodes };

            const newCtxMap = new Map(ctxMap);
            newCtxMap.set(currentCtxId, newContext);
            return newCtxMap;
        });

        if (activeAdapter) {
            try {
                const updatedContext = get(contexts).get(currentCtxId);
                if (updatedContext) {
                    await activeAdapter.saveContext(updatedContext);
                }
            } catch (error) {
                console.error(`[updateViewNodeAttribute] Error saving context ${currentCtxId} for ViewNode update:`, error);
            }
        }
    }

    // --- Update DataNode (Global Default for this node instance) ---
    if (needsDataNodeUpdate) { // This condition now correctly checks if a view_ or viewtype_ attribute changed
        nodes.update(nodeMap => {
            const originalDataNode = nodeMap.get(dataNodeId); // dataNode is already available from outer scope
            if (!originalDataNode) return nodeMap;

            const newDataAttributes = {
                ...(originalDataNode.attributes ?? {}),
                [attributeKey]: attributeValue // Update the specific view_ or viewtype_ attribute
            };

            const updatedDataNode: DataNode = {
                ...originalDataNode,
                attributes: newDataAttributes,
                modifiedAt: Date.now()
            };

            const newNodeMap = new Map(nodeMap);
            newNodeMap.set(dataNodeId, updatedDataNode);
            return newNodeMap;
        });

        // Persist DataNode changes to server
        if (activeAdapter) {
            try {
                const updatedDataNode = get(nodes).get(dataNodeId);
                if (updatedDataNode) {
                    await activeAdapter.saveNode(updatedDataNode);
                }
            } catch (error) {
                console.error(`[updateViewNodeAttribute] Error saving DataNode ${dataNodeId} for attribute default update:`, error);
            }
        }
    }
}

// --- Immediate UI Update (No Server Save) ---
export function updateViewNodeAttributeImmediate(viewNodeId: string, attributeKey: string, attributeValue: any): void {
    const currentCtxId = get(currentContextId);
    if (!currentCtxId) {
        return;
    }

    const currentContexts = get(contexts);
    const currentContext = currentContexts.get(currentCtxId);
    if (!currentContext) {
        return;
    }

    const viewNode = currentContext.viewNodes.get(viewNodeId);
    if (!viewNode) {
        return;
    }

    // Type check
    if (attributeKey.startsWith('type_')) {
        return;
    }

    // Only update ViewNode in context (no server save)
    const needsViewNodeUpdate = viewNode.attributes?.[attributeKey] !== attributeValue;
    
    if (needsViewNodeUpdate) {
        contexts.update(ctxMap => {
            const originalContext = ctxMap.get(currentCtxId);
            if (!originalContext) return ctxMap;
            const originalViewNode = originalContext.viewNodes.get(viewNodeId);
            if (!originalViewNode) return ctxMap;

            const newViewAttributes = { ...(originalViewNode.attributes ?? {}), [attributeKey]: attributeValue };
            const newViewNode = { ...originalViewNode, attributes: newViewAttributes };
            const newViewNodes = new Map(originalContext.viewNodes).set(viewNodeId, newViewNode);
            const newContext = { ...originalContext, viewNodes: newViewNodes };

            const newCtxMap = new Map(ctxMap);
            newCtxMap.set(currentCtxId, newContext);
            return newCtxMap;
        });
    }
}
// --- Node Search Action ---

/**
 * Adds an existing DataNode to the current context as a ViewNode.
 * If a ViewNode for this DataNode already exists, it selects it instead.
 * Fetches the DataNode by path.
 * Initializes the new ViewNode using default attributes from the DataNode.
 *
 * @param path The path of the DataNode to add.
 * @param position The canvas coordinates where the new ViewNode should be placed.
 */
export async function addExistingNodeToCurrentContext(path: string, position: { x: number; y: number }): Promise<void> {

    // 1. Check localAdapter
    if (!localAdapter) {
        console.error("[addExistingNodeToCurrentContext] LocalAdapter not available.");
        return;
    }

    try {
        // 2. Call localAdapter.getDataNodeByPath(path)
        const dataNode = await localAdapter.getDataNodeByPath(path);

        // 3. Check if DataNode exists
        if (!dataNode) {
            console.error(`[addExistingNodeToCurrentContext] DataNode with path "${path}" not found.`);
            // TODO: Provide user feedback? Maybe via a notification store?
            return;
        }

        // 4. Add the fetched DataNode to the global store if it's not already there
        // This prevents it from appearing as a ghost node
        nodes.update(n => {
            if (!n.has(dataNode.id)) {
                n.set(dataNode.id, dataNode);
            }
            return n;
        });

        // 5. Get current context ID and context
        const currentCtxId = get(currentContextId);
        if (!currentCtxId) {
            console.error("[addExistingNodeToCurrentContext] Cannot add node: No current context ID");
            return;
        }
        const currentContexts = get(contexts);
        const currentContext = currentContexts.get(currentCtxId);
        if (!currentContext) {
            console.error(`[addExistingNodeToCurrentContext] Cannot add node: Context ${currentCtxId} not found`);
            return;
        }

        // 6. Check if ViewNode for dataNode.id already exists in context.viewNodes
        if (currentContext.viewNodes.has(dataNode.id)) {
            // 7. If yes: select the existing ViewNode and center the view on it
            setSelectedNodes(new Set([dataNode.id]));

            // Center view on the existing node
            const existingViewNode = currentContext.viewNodes.get(dataNode.id);
            if (existingViewNode) {
                const nodeState = existingViewNode.state.current;
                // Calculate center based on node position and dimensions (assuming center anchor)
                const centerX = nodeState.x; // Already center X
                const centerY = nodeState.y; // Already center Y
                // If anchor wasn't center, calculation would be:
                // const centerX = nodeState.x + (nodeState.width / 2) * nodeState.scale;
                // const centerY = nodeState.y + (nodeState.height / 2) * nodeState.scale;
                centerViewOnCanvasPoint(centerX, centerY);
            } else {
                console.warn(`[addExistingNodeToCurrentContext] Could not find existing ViewNode ${dataNode.id} in context map to center view.`);
            }

            return; // Node already present, nothing more to do here
        }

        // 8. If no: Create a new ViewNode

        // 8a. Create initial state (using position and defaults from registry)
        const defaultViewState = getDefaultViewNodeStateForType(dataNode.ntype);
        const initialState: TweenableNodeState = {
            x: position.x,
            y: position.y,
            width: dataNode.attributes.view_width ?? defaultViewState.width, // Prioritize DataNode's stored default
            height: dataNode.attributes.view_height ?? defaultViewState.height, // Prioritize DataNode's stored default
            scale: defaultViewState.scale, // Scale is not yet a view_ attribute
            rotation: defaultViewState.rotation // Rotation is not yet a view_ attribute
        };

        // 8b. Create new ViewNode with Tween, copying relevant view-related default attributes from DataNode
        const viewAttributes: Record<string, any> = {};
        if (dataNode.attributes) {
            for (const key in dataNode.attributes) {
                // Copy view-related defaults (now prefixed with view_ or viewtype_)
                // to the ViewNode's attributes.
                if (key.startsWith('view_') || key.startsWith('viewtype_')) {
                    viewAttributes[key] = dataNode.attributes[key];
                }
            }
        }

        const newViewNode: ViewNode = {
            id: dataNode.id,
            state: new Tween(initialState, { duration: 0 }), // Initialize instantly
            attributes: viewAttributes,
            status: 'modified'
        };

        // 8c. Update contexts store immutably
        let updatedContext: Context | undefined = undefined;
        contexts.update(ctxMap => {
            const originalContext = ctxMap.get(currentCtxId);
            if (!originalContext) return ctxMap; // Should not happen

            // Create new viewNodes map immutably
            const newViewNodes = new Map(originalContext.viewNodes);
            newViewNodes.set(dataNode.id, newViewNode);

            // Create new context object immutably
            updatedContext = { // Assign to outer variable
                ...originalContext,
                viewNodes: newViewNodes,
                // Capture current viewport settings when adding a node? Or rely on context switch save?
                // Let's capture it here for consistency with createNodeAtPosition
                viewportSettings: { ...viewTransform.current }
            };

            // Create new top-level map immutably
            const newCtxMap = new Map(ctxMap);
            newCtxMap.set(currentCtxId, updatedContext);
            return newCtxMap;
        });

        // 8d. Persist context via localAdapter.saveContext()
        if (updatedContext) {
            await localAdapter.saveContext(updatedContext);
            // Optionally select the newly added node
            setSelectedNodes(new Set([dataNode.id]));
        } else {
            console.error(`[addExistingNodeToCurrentContext] Failed to get updated context ${currentCtxId} for saving after adding ViewNode.`);
            // TODO: Consider rolling back store update?
        }

    } catch (error) {
        console.error(`[addExistingNodeToCurrentContext] Error adding node with path "${path}":`, error);
    }
}