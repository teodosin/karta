import { writable, get } from 'svelte/store';
import { localAdapter } from '../util/LocalAdapter';
import type { DataNode, NodeId, AssetData, AbsoluteTransform, ViewportSettings, TweenableNodeState, StorableContext, StorableViewNode, StorableViewportSettings, ViewNode, Context } from '../types/types'; // Added Context type
import { v4 as uuidv4 } from 'uuid';
import { getDefaultAttributesForType, getDefaultViewNodeStateForType } from '$lib/node_types/registry';
import { Tween } from 'svelte/motion';
// Import removeViewNodeFromContext as well
import { currentContextId, contexts, ROOT_NODE_ID, removeViewNodeFromContext } from './ContextStore';
import { viewTransform } from './ViewportStore'; // Assuming ViewportStore will export this

export const nodes = writable<Map<NodeId, DataNode>>(new Map());

async function _ensureDataNodeExists(nodeId: NodeId): Promise<DataNode | null> {
    if (!localAdapter) {
        console.error(`[_ensureDataNodeExists] LocalAdapter not available while checking for ${nodeId}`);
        return null;
    }
    try {
        let dataNode = await localAdapter.getNode(nodeId);
        if (nodeId === ROOT_NODE_ID) {
            console.log(`[_ensureDataNodeExists] Root node check: Found in DB?`, !!dataNode, dataNode ? `Existing ntype: ${dataNode.ntype}` : '');
        }
        if (!dataNode) {
            console.warn(`[_ensureDataNodeExists] DataNode ${nodeId} not found. Creating default.`);
            const now = Date.now();
            // Determine default properties based on whether it's the root node
            const isRoot = nodeId === ROOT_NODE_ID;
            const defaultName = isRoot ? 'root' : `node-${nodeId.substring(0, 8)}`;
            const defaultPath = isRoot ? '/root' : `/${defaultName}`;
            const defaultNtype = isRoot ? 'root' : 'generic';
            if (isRoot) {
                console.log(`[_ensureDataNodeExists] Root node creation: Assigning ntype: ${defaultNtype}`);
            }

            dataNode = {
                id: nodeId,
                ntype: defaultNtype, // Set type correctly
                createdAt: now,
                modifiedAt: now,
                path: defaultPath, // Set path correctly
                attributes: { name: defaultName, ...(isRoot && { isSystemNode: true }) }, // Set name and system flag for root
            };
            await localAdapter.saveNode(dataNode);
            console.log(`[_ensureDataNodeExists] Default DataNode ${nodeId} created and saved.`);
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
	ntype: string = 'text',
	attributes: Record<string, any> = {},
	initialWidth?: number, // Optional initial width
	initialHeight?: number // Optional initial height
): Promise<NodeId | null> {
	const newNodeId: NodeId = uuidv4();
    const now = Date.now();
    let baseName = attributes.name || ntype;
    let finalName = baseName;
    let counter = 2;
    // Ensure name uniqueness (assuming checkNameExists works correctly)
    if (localAdapter) {
        while (await localAdapter.checkNameExists(finalName)) {
            finalName = `${baseName}${counter}`; counter++;
        }
    } else { console.warn("LocalAdapter not ready, cannot check for duplicate names."); }

	// 1. Create DataNode
	const newNodeData: DataNode = {
		id: newNodeId, ntype: ntype, createdAt: now, modifiedAt: now,
        path: `/${finalName}`, // Simple path for now
		attributes: { ...attributes, name: finalName },
	};

    // 2. Get default view state based on ntype and create initial state for the ViewNode's tween
    const defaultViewState = getDefaultViewNodeStateForType(ntype); // Gets { width, height, scale, rotation }
    const initialState: TweenableNodeState = {
        x: canvasX,
        y: canvasY,
        width: initialWidth ?? defaultViewState.width, // Use provided width or default
        height: initialHeight ?? defaultViewState.height, // Use provided height or default
        scale: defaultViewState.scale, // Keep default scale
        rotation: defaultViewState.rotation // Keep default rotation
    };

    // 3. Create the new ViewNode containing the Tween
    const newViewNode: ViewNode = {
        id: newNodeId,
        state: new Tween(initialState, { duration: 0 }) // Initialize instantly, no animation
    };

    // 4. Update stores
    const contextId = get(currentContextId);
	nodes.update(n => n.set(newNodeId, newNodeData)); // Add DataNode

    contexts.update((ctxMap: Map<NodeId, Context>) => { // Explicitly type ctxMap
        let currentCtx = ctxMap.get(contextId);
        if (!currentCtx) {
            console.warn(`Context ${contextId} not found when creating node ${newNodeId}. Creating context.`);
            currentCtx = { id: contextId, viewNodes: new Map() };
            ctxMap.set(contextId, currentCtx); // Add new context to map if needed
        }
        currentCtx.viewNodes.set(newNodeId, newViewNode); // Add new ViewNode to context
        return ctxMap; // Return the modified map to trigger update
    });

    // 5. Persist changes
    if (localAdapter) {
        try {
            await localAdapter.saveNode(newNodeData);
            const updatedCtx = get(contexts).get(contextId);
            if (updatedCtx) {
                updatedCtx.viewportSettings = { ...viewTransform.current }; // Capture viewport state - Access .current directly
                await localAdapter.saveContext(updatedCtx); // Save context
            }
            console.log(`[KartaStore] Node ${newNodeId} and Context ${contextId} saved.`);
            return newNodeId; // Return ID on successful save
        } catch (error) {
            console.error("Error saving node or context after creation:", error);
            // Optionally remove the node from stores if save failed? For now, just return null.
            return null;
        }
    } else {
        console.warn("LocalAdapter not initialized, persistence disabled.");
        // Still add to stores for non-persistent use, but return ID
        return newNodeId;
        // Or return null if persistence is strictly required? Let's return ID for now.
    }
}

export async function createImageNodeFromDataUrl(position: { x: number, y: number }, dataUrl: string, width?: number, height?: number) {
	try {
		// Create the basic node structure first
		// Pass optional width and height to createNodeAtPosition
		const newNodeId = await createNodeAtPosition(position.x, position.y, 'image', {}, width, height);
		if (!newNodeId) {
			console.error("[KartaStore] Failed to create base node for image paste.");
			return;
		}

		// Update the attributes with the image source
		// TODO: Consider adding warnings or size limits for very large images.
		await updateNodeAttributes(newNodeId, { src: dataUrl });

		console.log(`[KartaStore] Created ImageNode ${newNodeId} from Data URL at (${position.x}, ${position.y})`);
	} catch (error) {
		console.error("[KartaStore] Error creating image node from Data URL:", error);
	}
}

export async function createTextNodeFromPaste(position: { x: number, y: number }, text: string) {
	try {
		// Create the basic node structure first
		const newNodeId = await createNodeAtPosition(position.x, position.y, 'text');
		if (!newNodeId) {
			console.error("[KartaStore] Failed to create base node for text paste.");
			return;
		}

		// Update the attributes with the pasted text
		await updateNodeAttributes(newNodeId, { text: text });

		console.log(`[KartaStore] Created TextNode ${newNodeId} from paste at (${position.x}, ${position.y})`);
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
    if (!localAdapter) {
        console.error("[createImageNodeWithAsset] LocalAdapter not available.");
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
            'image',
            { alt: assetName }, // Set alt attribute initially
            initialWidth,
            initialHeight
        );

        if (!newNodeId) {
            throw new Error("Failed to create base node structure.");
        }

        // 2. Prepare asset data and save it using the newNodeId as assetId
        const assetData: AssetData = { blob: imageBlob, mimeType: imageBlob.type, name: assetName };
        await localAdapter.saveAsset(newNodeId, assetData);

        // 3. Update the node attributes with the Object URL and assetId
        // Use the existing updateNodeAttributes function
        await updateNodeAttributes(newNodeId, { src: objectUrl, assetId: newNodeId });

        console.log(`[KartaStore] Created ImageNode ${newNodeId} with asset ${assetName}`);
        return newNodeId;

    } catch (error) {
        console.error("[createImageNodeWithAsset] Error:", error);
        // Cleanup logic:
        if (newNodeId) {
            // If the base node was created, attempt to delete it from DB.
            // The updated localAdapter.deleteNode will also call deleteAsset,
            // which handles revoking any URL *it* might have tracked.
            console.log(`[createImageNodeWithAsset] Cleaning up partially created node ${newNodeId}`);
            await localAdapter.deleteNode(newNodeId);
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
        console.log(`[createImageNodeWithAsset] Revoked initial objectUrl: ${objectUrl}`);

        return null; // Indicate failure
    }
}


export async function updateNodeAttributes(nodeId: NodeId, newAttributes: Record<string, any>) {
    const currentNodes = get(nodes);
    const dataNode = currentNodes.get(nodeId);

    if (!dataNode) {
        console.warn(`[updateNodeAttributes] DataNode ${nodeId} not found in store.`);
        return;
    }

    // Prevent renaming system nodes
    if (dataNode.attributes?.isSystemNode) {
        console.warn(`[updateNodeAttributes] Attempted to modify attributes of system node ${nodeId}. Operation cancelled.`);
        return;
    }

    const oldName = dataNode.attributes?.name;
    const newName = newAttributes?.name;
    let attributesToSave = { ...dataNode.attributes, ...newAttributes }; // Merge old and new

    // Check for name change and uniqueness
    if (newName && newName.trim() && newName !== oldName) {
        const finalNewName = newName.trim();
        if (localAdapter) {
            let nameToSet = finalNewName; // Start with the user's desired name
            let nameExists = await localAdapter.checkNameExists(nameToSet);

            if (nameExists) {
                console.warn(`[updateNodeAttributes] Name "${nameToSet}" already exists. Finding next available name...`);
                const baseName = nameToSet; // Store the original desired name
                let counter = 2;
                // Loop until a unique name is found
                while (nameExists) {
                    nameToSet = `${baseName}${counter}`;
                    nameExists = await localAdapter.checkNameExists(nameToSet);
                    counter++;
                }
                console.log(`[updateNodeAttributes] Unique name found: "${nameToSet}"`);
            }
            // Update attributesToSave with the final unique name
            attributesToSave.name = nameToSet;
        } else {
            console.warn("[updateNodeAttributes] LocalAdapter not ready, cannot check for duplicate names.");
            // If adapter isn't ready, we cannot guarantee uniqueness. Cancel the rename.
            console.error("[updateNodeAttributes] LocalAdapter not available. Cannot verify name uniqueness. Rename cancelled.");
            return;
        }
    } else if (newName !== undefined && !newName.trim()) {
        console.warn(`[updateNodeAttributes] Attempted to rename node ${nodeId} to an empty name. Operation cancelled.`);
        return; // Prevent empty names
    }

    // Create updated node data
    // Create updated node data
    const updatedNodeData: DataNode = {
        ...dataNode,
        attributes: attributesToSave, // Use the final attributes map (name might have been incremented)
        modifiedAt: Date.now(),
        // Potentially update path if name changed? For now, keep path separate.
        // path: `/${attributesToSave.name}` // Example if path should sync
    };

    // Update the store
    // Update the store only if changes were actually made
    // Compare the final attributesToSave with the original dataNode attributes
    if (JSON.stringify(attributesToSave) !== JSON.stringify(dataNode.attributes)) {
        nodes.update(n => n.set(nodeId, updatedNodeData));
        console.log(`[updateNodeAttributes] Updated attributes for node ${nodeId}:`, attributesToSave);

        // Persist changes
        if (localAdapter) {
            try {
                await localAdapter.saveNode(updatedNodeData);
            } catch (error) {
                console.error(`[updateNodeAttributes] Error saving node ${nodeId} after attribute update:`, error);
                // Optionally revert store update on save failure?
                // nodes.update(n => n.set(nodeId, dataNode)); // Example revert
            }
        } else {
            console.warn("[updateNodeAttributes] LocalAdapter not initialized, persistence disabled.");
        }
    } else {
        console.log(`[updateNodeAttributes] No effective attribute changes for node ${nodeId}.`);
    }
   }


export async function fetchAvailableContextDetails(): Promise<{ id: NodeId, name: string, path: string }[]> {
    if (!localAdapter) {
        console.error("[fetchAvailableContextDetails] LocalAdapter not available.");
        return [];
    }
    try {
        const contextIds = await localAdapter.getAllContextIds();
        if (contextIds.length === 0) {
            return [];
        }

        const dataNodesMap = await localAdapter.getDataNodesByIds(contextIds);
        const contextDetails = Array.from(dataNodesMap.values())
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
   import { edges, deleteEdge } from './EdgeStore'; // Assuming EdgeStore exports edges store and deleteEdge action
   
   export async function deleteDataNodePermanently(nodeId: NodeId): Promise<void> {
    console.log(`[NodeStore] Attempting permanent deletion for node: ${nodeId}`);
    if (!localAdapter) {
    	console.error("[deleteDataNodePermanently] LocalAdapter not available.");
    	return;
    }
   
    try {
    	// 1. Get the node data to check type for asset deletion
    	const dataNodeToDelete = get(nodes).get(nodeId); // Get from store first
    	if (!dataNodeToDelete) {
    		// If not in store, try fetching from DB (might be a ghost node scenario)
    		const nodeFromDb = await localAdapter.getNode(nodeId);
    		if (!nodeFromDb) {
    			console.warn(`[deleteDataNodePermanently] Node ${nodeId} not found in store or DB. Cannot delete.`);
    			return;
    		}
    		// If found in DB but not store, proceed with deletion from DB
    		console.warn(`[deleteDataNodePermanently] Node ${nodeId} found in DB but not in store. Proceeding with DB deletion.`);
    	}
   
    	// 2. Find connected edges
    	const allEdges = get(edges);
    	const connectedEdges = [...allEdges.values()].filter(edge => edge.source === nodeId || edge.target === nodeId);
    	console.log(`[deleteDataNodePermanently] Found ${connectedEdges.length} connected edges for node ${nodeId}.`);
   
    	// 3. Remove DataNode from the store *first* to trigger UI updates (like ghosting)
    	let nodeRemovedFromStore = false;
    	nodes.update(n => {
    		if (n.has(nodeId)) {
    			n.delete(nodeId);
    			nodeRemovedFromStore = true;
    			return n;
    		}
    		return n; // Return original map if node wasn't there
    	});
    	if (nodeRemovedFromStore) {
    		console.log(`[NodeStore] Removed node ${nodeId} from store.`);
    	}
   
   
    	// 4. Delete connected edges (store update + persistence handled by deleteEdge)
    	for (const edge of connectedEdges) {
    		await deleteEdge(edge.id); // deleteEdge should handle store and persistence
    	}
    	console.log(`[deleteDataNodePermanently] Deleted ${connectedEdges.length} connected edges for node ${nodeId}.`);
   
    	// 5. Delete asset if it's an image node
    	// Use the node data we fetched earlier (either from store or DB)
    	const nodeType = dataNodeToDelete?.ntype ?? (await localAdapter.getNode(nodeId))?.ntype; // Check type
    	if (nodeType === 'image') {
    		// Asset ID is the same as Node ID for images currently
    		await localAdapter.deleteAsset(nodeId);
    		console.log(`[deleteDataNodePermanently] Deleted associated asset for image node ${nodeId}.`);
    	}
   
    	// 6. Delete the DataNode from persistence
    	await localAdapter.deleteNode(nodeId);
    	console.log(`[deleteDataNodePermanently] Successfully deleted node ${nodeId} from persistence.`);
   
    	// 7. Remove the ViewNode from the *current* context
    	const currentCtxId = get(currentContextId);
    	if (currentCtxId) {
    		await removeViewNodeFromContext(currentCtxId, nodeId);
    		console.log(`[deleteDataNodePermanently] Removed ViewNode ${nodeId} from current context ${currentCtxId}.`);
    	} else {
    		console.warn(`[deleteDataNodePermanently] Could not determine current context ID to remove ViewNode ${nodeId}.`);
    	}
   
   
    } catch (error) {
    	console.error(`[deleteDataNodePermanently] Error deleting node ${nodeId}:`, error);
    	// Consider adding rollback logic if needed, though complex.
    	// For now, log the error. The node might be partially deleted.
    }
   }
   
   
   export { _ensureDataNodeExists };