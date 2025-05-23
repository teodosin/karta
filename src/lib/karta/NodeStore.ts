import { writable, get } from 'svelte/store';
import { setSelectedNodes } from './SelectionStore'; // Import selection action
// Removed highlightNodeId import
import { localAdapter } from '../util/LocalAdapter';
import type { DataNode, NodeId, AssetData, AbsoluteTransform, ViewportSettings, TweenableNodeState, StorableContext, StorableViewNode, StorableViewportSettings, ViewNode, Context } from '../types/types'; // Added Context type
import { v4 as uuidv4 } from 'uuid';
import { getDefaultAttributesForType, getDefaultViewNodeStateForType } from '$lib/node_types/registry';
import { Tween } from 'svelte/motion';
// Import removeViewNodeFromContext and availableContextsMap as well
import { currentContextId, contexts, ROOT_NODE_ID, removeViewNodeFromContext, availableContextsMap } from './ContextStore';
import { viewTransform, centerViewOnCanvasPoint } from './ViewportStore'; // Import centering function

export const nodes = writable<Map<NodeId, DataNode>>(new Map());

async function _ensureDataNodeExists(nodeId: NodeId): Promise<DataNode | null> {
    if (!localAdapter) {
        console.error(`[_ensureDataNodeExists] LocalAdapter not available while checking for ${nodeId}`);
        return null;
    }
    try {
        let dataNode = await localAdapter.getNode(nodeId);
        if (nodeId === ROOT_NODE_ID) {
        }
        if (!dataNode) {
            console.warn(`[_ensureDataNodeExists] DataNode ${nodeId} not found. Creating default.`);
            const now = Date.now();
            // Determine default properties based on whether it's the root node
            const isRoot = nodeId === ROOT_NODE_ID;
            const defaultName = isRoot ? 'root' : `node-${nodeId.substring(0, 8)}`;
            const defaultPath = isRoot ? '/root' : `/${defaultName}`;
            const defaultNtype = isRoot ? 'root' : 'node';
            if (isRoot) {
            }

            dataNode = {
                id: nodeId,
                ntype: defaultNtype, // Set type correctly
                createdAt: now,
                modifiedAt: now,
                path: defaultPath, // Set path correctly
                attributes: { name: defaultName, ...(isRoot && { isSystemNode: true, view_isNameVisible: false }) }, // Set name, system flag, and hide label for root
            };
            await localAdapter.saveNode(dataNode);
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
		attributes: {
		          ...getDefaultAttributesForType(ntype), // Get new prefixed defaults
		          ...attributes, // Apply user-provided attributes (can override defaults)
		          name: finalName // Ensure name is set (unprefixed)
		      },
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
            { alt: assetName, name: assetName }, // Set alt and name attributes initially
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

        return newNodeId;

    } catch (error) {
        console.error("[createImageNodeWithAsset] Error:", error);
        // Cleanup logic:
        if (newNodeId) {
            // If the base node was created, attempt to delete it from DB.
            // The updated localAdapter.deleteNode will also call deleteAsset,
            // which handles revoking any URL *it* might have tracked.
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
        // Update path if name changed
        path: `/${attributesToSave.name}`
    };

    // Update the store
    // Update the store only if changes were actually made
    // Compare the final attributesToSave with the original dataNode attributes
    if (JSON.stringify(attributesToSave) !== JSON.stringify(dataNode.attributes)) {
        nodes.update(n => n.set(nodeId, updatedNodeData));

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
    }
   }
export async function updateNodeSearchableFlag(nodeId: NodeId, isSearchable: boolean): Promise<void> {
    const currentNodes = get(nodes);
    const dataNode = currentNodes.get(nodeId);

    if (!dataNode) {
        console.warn(`[updateNodeSearchableFlag] DataNode ${nodeId} not found in store.`);
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
    if (localAdapter) {
        try {
            await localAdapter.saveNode(updatedNodeData);
        } catch (error) {
            console.error(`[updateNodeSearchableFlag] Error saving node ${nodeId} after searchable flag update:`, error);
            // Optionally revert store update on save failure?
            // nodes.update(n => n.set(nodeId, dataNode)); // Example revert
        }
    } else {
        console.warn("[updateNodeSearchableFlag] LocalAdapter not initialized, persistence disabled.");
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
   import { edges, deleteEdge } from './EdgeStore'; // Assuming EdgeStore exports edges store and deleteEdge action
   
   export async function deleteDataNodePermanently(nodeId: NodeId): Promise<void> {
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
    	}
   
   
    	// 4. Delete connected edges (store update + persistence handled by deleteEdge)
    	for (const edge of connectedEdges) {
    		await deleteEdge(edge.id); // deleteEdge should handle store and persistence
    	}
   
    	// 5. Delete asset if it's an image node
    	// Use the node data we fetched earlier (either from store or DB)
    	const nodeType = dataNodeToDelete?.ntype ?? (await localAdapter.getNode(nodeId))?.ntype; // Check type
    	if (nodeType === 'image') {
    		// Asset ID is the same as Node ID for images currently
    		await localAdapter.deleteAsset(nodeId);
    	}
   
    	// 6. Delete the DataNode from persistence
    	await localAdapter.deleteNode(nodeId);
    
    	// 7. Delete the corresponding Context (if it exists) and update the map
    	try {
    		await localAdapter.deleteContext(nodeId);
    		// If context deletion was successful, remove from the map
    		availableContextsMap.update(map => {
    			if (map.has(nodeId)) {
    				map.delete(nodeId);
    				console.log(`[NodeStore] Removed context ${nodeId} from availableContextsMap.`);
    			}
    			return map;
    		});
    	} catch (contextDeleteError) {
    		// Log error, but continue node deletion process
    		console.error(`[deleteDataNodePermanently] Error deleting context for node ${nodeId}:`, contextDeleteError);
    	}
    
    	// 8. Remove the ViewNode from the *current* context
    	const currentCtxId = get(currentContextId);
    	if (currentCtxId) {
    		await removeViewNodeFromContext(currentCtxId, nodeId);
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
       // Only check ViewNode for updates, as DataNode attributes (defaults) are not changed here.
       const needsViewNodeUpdate = viewNode.attributes?.[attributeKey] !== attributeValue;
 
       // --- Trigger Store Updates & Persistence (using immutable patterns) ---
       if (needsViewNodeUpdate) {
           contexts.update(ctxMap => {
               // Get the original context and viewNodes map again inside update
               const originalContext = ctxMap.get(currentCtxId);
               if (!originalContext) return ctxMap; // Should not happen based on earlier checks, but safe
               const originalViewNode = originalContext.viewNodes.get(viewNodeId);
               if (!originalViewNode) return ctxMap; // Should not happen

               // Create new attributes object immutably
               const newViewAttributes = {
                   ...(originalViewNode.attributes ?? {}), // Ensure attributes object exists
                   [attributeKey]: attributeValue
               };

               // Create new ViewNode object immutably
               const newViewNode = {
                   ...originalViewNode,
                   attributes: newViewAttributes
               };

               // Create new viewNodes map immutably
               const newViewNodes = new Map(originalContext.viewNodes);
               newViewNodes.set(viewNodeId, newViewNode);

               // Create new context object immutably
               const newContext = {
                   ...originalContext,
                   viewNodes: newViewNodes
               };

               // Create new top-level map immutably
               const newCtxMap = new Map(ctxMap);
               newCtxMap.set(currentCtxId, newContext);
               return newCtxMap;
           });

           if (localAdapter) {
               try {
                   // Save the entire context as ViewNode attributes changed
                   const updatedContext = get(contexts).get(currentCtxId);
                   if (updatedContext) {
                       await localAdapter.saveContext(updatedContext);
                   } else {
                        console.error(`[updateViewNodeAttribute] Failed to get updated context ${currentCtxId} for saving.`);
                   }
               } catch (error) {
                   console.error(`[updateViewNodeAttribute] Error saving context ${currentCtxId} after view attribute update:`, error);
                   // Consider rollback? For now, just log.
               }
           }
       }
 
       // Only log if a ViewNode update occurred.
       if (needsViewNodeUpdate) {
       } else {
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
            width: defaultViewState.width,
            height: defaultViewState.height,
            scale: defaultViewState.scale,
            rotation: defaultViewState.rotation
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
            attributes: viewAttributes
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