import { writable, get, derived } from 'svelte/store';
import { apiLogger, storeLogger } from '$lib/debug';
import { Tween } from 'svelte/motion';
import { cubicInOut, cubicOut } from 'svelte/easing';
import { LocalAdapter, localAdapter } from '../util/LocalAdapter';
import { ServerAdapter } from '../util/ServerAdapter';
import type { PersistenceService } from '../util/PersistenceService';
import type { 
    DataNode, KartaEdge, ViewNode, Context, NodeId, EdgeId,
    AbsoluteTransform, ViewportSettings, TweenableNodeState, StorableContext,
    StorableViewNode, StorableViewportSettings, KartaExportData, ContextBundle, 
	NodePath,
	NodeHandle
} from '../types/types';
import { getDefaultViewNodeStateForType } from '$lib/node_types/registry';
import { nodes, _ensureDataNodeExists } from './NodeStore';
import { edges } from './EdgeStore';
import { 
    viewTransform, 
    DEFAULT_VIEWPORT_SETTINGS, VIEWPORT_TWEEN_DURATION, DEFAULT_FOCAL_TRANSFORM, 
    centerOnFocalNode 
} from './ViewportStore';
import { historyStack, futureStack } from './HistoryStore';
import { clearSelection } from './SelectionStore';
import { propertiesPanelPosition } from './UIStateStore';
import { setTool } from './ToolStore';
import { settings, updateSettings } from './SettingsStore';
import { ROOT_NODE_ID } from '$lib/constants';



const USE_SERVER_ADAPTER = true;
export let activeAdapter: PersistenceService;


if (USE_SERVER_ADAPTER) {
    activeAdapter = new ServerAdapter();
    console.log('[ContextStore] Using ServerAdapter');
} else {
    if (localAdapter) {
        activeAdapter = localAdapter;
        console.log('[ContextStore] Using LocalAdapter');
    } else {
        // This case should ideally not happen if LocalAdapter is crucial for non-server mode.
        // Or, we need a NoopAdapter. For now, activeAdapter might be undefined.
        console.error("[ContextStore] LocalAdapter is null. Persistence will not work in local mode.");
        // activeAdapter remains undefined, subsequent checks for activeAdapter will handle this.
    }
}


const NODE_TWEEN_DURATION = 1000; // TODO: Make this configurable in settings
const NODE_TWEEN_OPTIONS = { duration: NODE_TWEEN_DURATION, easing: cubicInOut };


export const contexts = writable<Map<NodeId, Context>>(new Map());
export const currentContextId = writable<NodeId>(ROOT_NODE_ID);

// Store for available contexts (ID -> Path)
export const existingContextsMap = writable<Map<NodeId, string>>(new Map());

// Derived Store for Current Context's ViewNodes
export const currentViewNodes = derived(
    [currentContextId, contexts],
    ([$currentContextId, $contexts]) => {
        return $contexts.get($currentContextId)?.viewNodes ?? new Map<NodeId, ViewNode>();
    }
);





/* 
    Fetches the initial state for the focal node in a new context. 
*/
async function _getFocalNodeInitialState(targetNodeId: NodeId, oldContext: Context | undefined): Promise<TweenableNodeState> {
    const targetViewNodeInOldCtx = oldContext?.viewNodes.get(targetNodeId);

    if (targetViewNodeInOldCtx) {
        const state = { ...targetViewNodeInOldCtx.state.current };
        return state;

    } else {
        const dataNode = await _ensureDataNodeExists(targetNodeId);
        const defaultState: { 
            width: number; height: number; scale: number; rotation: number; 
        } = dataNode ? getDefaultViewNodeStateForType(dataNode.ntype) : getDefaultViewNodeStateForType('core/generic');

        const finalDefaultState = {
            ...DEFAULT_FOCAL_TRANSFORM,
            width: defaultState.width,
            height: defaultState.height,
            rotation: defaultState.rotation
        };
        return finalDefaultState;
    }
}



/**
 * Takes a raw `StorableContext` from a persistence bundle and processes it into a live,
 * in-memory `Context` object. This involves:
 * - Converting `StorableViewNode` objects into `ViewNode` objects with active Svelte tweens for animation.
 * - Reusing existing tweens from the `oldContext` for smooth transitions.
 * - Creating a new context from scratch if one doesn't exist in storage.
 * - Adding default connected nodes that might not have been persisted in the last save.
 * - Saving the newly created context to persistence immediately.
 *
 * This function is the core of merging persisted data with the live application state.
 */
async function _loadAndProcessContext(
    contextId: NodeId,
    focalInitialStateFromOldContext: TweenableNodeState,
    oldContext: Context | undefined, // Previous context for potential tween reuse
    storableContext: StorableContext | undefined
): Promise<{ finalContext: Context | undefined, wasCreated: boolean }> {

    if (!activeAdapter) return { finalContext: undefined, wasCreated: false };

    let contextWasCreated = false;
    const finalViewNodes = new Map<NodeId, ViewNode>();
    let finalViewportSettings: ViewportSettings | undefined = undefined;

    if (storableContext) {
        // --- Context exists in DB ---
        // Convert StorableViewNodes to ViewNodes with Tweens, reusing old tweens if possible
        for (const [nodeId, storableNode] of storableContext.viewNodes) {
            // Extract only the placement part needed by _calculateTargetState
            const focalPlacement: AbsoluteTransform = { x: focalInitialStateFromOldContext.x, y: focalInitialStateFromOldContext.y, scale: focalInitialStateFromOldContext.scale };
            const targetState = _calculateTargetState(nodeId, contextId, focalPlacement, storableNode);
            const existingViewNode = oldContext?.viewNodes.get(nodeId);

            if (existingViewNode) {
                console.log(`[ContextStore._loadAndProcessContext] Cloning existing ViewNode ${nodeId} from old context for storable node.`);
                finalViewNodes.set(nodeId, cloneViewNode(existingViewNode, targetState));
            } else {
                // Create new ViewNode with a new Tween, starting from target state? Or current if exists? Start from target.
                finalViewNodes.set(nodeId, { id: nodeId, state: new Tween(targetState, NODE_TWEEN_OPTIONS), attributes: storableNode.attributes, status: storableNode.status });
            }
        }
        // Convert stored viewport settings
        finalViewportSettings = _convertStorableViewportSettings(storableContext.viewportSettings, finalViewNodes.get(contextId));

    } else {
        // --- Context needs creation ---
        contextWasCreated = true;
        const focalDataNode = await activeAdapter.getNode(contextId);
        if (!focalDataNode) throw new Error(`Cannot create context for non-existent node: ${contextId}`);

        // Use the state passed in, which already contains defaults or state from old context
        const correctedFocalInitialState: TweenableNodeState = {
            x: focalInitialStateFromOldContext.x,
            y: focalInitialStateFromOldContext.y,
            scale: focalInitialStateFromOldContext.scale,
            rotation: focalInitialStateFromOldContext.rotation,
            width: focalInitialStateFromOldContext.width,
            height: focalInitialStateFromOldContext.height
        };
        finalViewNodes.set(contextId, { id: contextId, state: new Tween(correctedFocalInitialState, { duration: 0 }), status: 'modified' });
    }

    // --- Add Previous Focal Node (if context is new and applicable) ---
    if (contextWasCreated) {
        const currentHistory: NodeId[] = get(historyStack);
        const previousContextId = currentHistory.length > 0 ? currentHistory[currentHistory.length - 1] : null;
        if (previousContextId && previousContextId !== contextId && !finalViewNodes.has(previousContextId)) {
            const previousFocalViewNode = oldContext?.viewNodes.get(previousContextId);
            if (previousFocalViewNode) {
                console.log(`[ContextStore._loadAndProcessContext] Cloning previous focal node ${previousContextId} from old context.`);
                finalViewNodes.set(previousContextId, cloneViewNode(previousFocalViewNode));
            }
        }
    }


    const finalContext: Context = {
        id: contextId,
        viewNodes: finalViewNodes,
        viewportSettings: finalViewportSettings
    };


    if (contextWasCreated && activeAdapter) {
        try {
            await activeAdapter.saveContext(finalContext);

            const focalDataNode = get(nodes).get(contextId);
            if (focalDataNode?.path) {
                existingContextsMap.update(map => {
                    map.set(contextId, focalDataNode.path);
                    return map;
                });
            } else {
            }
        } catch (error) {
            console.error(`[ContextStore] Error saving newly created context ${contextId}:`, error);
            // Should we still return the context even if saving failed? Yes, probably.
        }
    }

    return { finalContext, wasCreated: contextWasCreated };
}





function _calculateTargetState(
    nodeId: NodeId,
    contextId: NodeId,
    focalPlacement: AbsoluteTransform,
    storableNode: StorableViewNode
): TweenableNodeState {
    if (nodeId === contextId) {

        return {
            x: focalPlacement.x,
			y: focalPlacement.y,
            scale: focalPlacement.scale,
			rotation: storableNode.rotation,
            width: storableNode.width,
			height: storableNode.height
        };

    } else {

        const absScale 		= focalPlacement.scale 	* storableNode.relScale;
        const scaledRelX 	= storableNode.relX 	* focalPlacement.scale;
        const scaledRelY 	= storableNode.relY 	* focalPlacement.scale;
        const absX 			= focalPlacement.x 		+ scaledRelX;
        const absY 			= focalPlacement.y 		+ scaledRelY;

        return {
            x: absX,
			y: absY,
			scale: absScale,
			rotation: storableNode.rotation,
            width: storableNode.width,
			height: storableNode.height
        };
    }
}





function _convertStorableViewportSettings(
    storableSettings: StorableViewportSettings | undefined,
    focalViewNode: ViewNode | undefined
): ViewportSettings | undefined {

    if (!storableSettings) {
        return undefined;
    }

    const focalState = focalViewNode?.state.current;

    if (focalState) {
        // Explicitly check for NaN before calculation
        if (
				isNaN(focalState.x) 
			|| 	isNaN(focalState.y) 
			|| 	isNaN(storableSettings.scale) 
			|| 	isNaN(storableSettings.relPosX) 
			|| 	isNaN(storableSettings.relPosY)
		) {
			storeLogger.error("CRITICAL: NaN in viewport settings conversion!",
                {
					x: focalState.x,
					y: focalState.y,
					scale: storableSettings.scale,
					relPosX: storableSettings.relPosX,
					relPosY: storableSettings.relPosY 
				}
            );

        }

        const absPosX = storableSettings.relPosX - (focalState.x * storableSettings.scale);
        const absPosY = storableSettings.relPosY - (focalState.y * storableSettings.scale);

        const result = { scale: storableSettings.scale, posX: absPosX, posY: absPosY };

        if (isNaN(absPosX) || isNaN(absPosY)) {
			storeLogger.error("CRITICAL: NaN in viewport settings conversion!", result);
        }

        return result;

    } else {

		storeLogger.warn("Focal node state not found. Returning DEFAULT_VIEWPORT_SETTINGS.");
        return { ...DEFAULT_VIEWPORT_SETTINGS };
    }
}





function _applyStoresUpdate(
    newContextId: NodeId,
    processedContext: Context,
    loadedDataNodesForContext: Map<NodeId, DataNode>,
    loadedEdgesForContext: Map<EdgeId, KartaEdge>
) {

    // --- Update nodes store (Merge) ---
    // Keep all existing nodes, only add/update the ones loaded for the new context
    nodes.update(currentNodesMap => {
        const nextNodes = new Map<NodeId, DataNode>(currentNodesMap);
        for (const [nodeId, dataNode] of loadedDataNodesForContext.entries()) {
            nextNodes.set(nodeId, dataNode);
        }
        return nextNodes;
    });

    // --- Update edges store (Filter based on new context's view) ---
    // Edges are still context-dependent for display
    const nextEdges = new Map<EdgeId, KartaEdge>();
    for (const [edgeId, edge] of loadedEdgesForContext.entries()) {
        // Only keep edges where both source and target nodes are present in the new context's view
        if (processedContext.viewNodes.has(edge.source) && processedContext.viewNodes.has(edge.target)) {
            nextEdges.set(edgeId, edge);
        }
    }
    edges.set(nextEdges);

    // --- Update contexts store ---
    // Set the fully processed context (containing ViewNodes with Tweens)
    contexts.update((map: Map<NodeId, Context>) => map.set(newContextId, processedContext)); // Explicitly type map

    // --- Update current context ID ---
    currentContextId.set(newContextId);
}





/**
 * Creates a deep copy of a ViewNode, particularly its tweened state, to prevent reference sharing across contexts.
 * @param viewNode The source ViewNode to clone.
 * @param newTargetState An optional new target state for the tween.
 */
function cloneViewNode(viewNode: ViewNode, newTargetState?: TweenableNodeState): ViewNode {
    // Deep copy the current state of the tween.
    const initialState = { ...viewNode.state.current };
    const newTween = new Tween(initialState, NODE_TWEEN_OPTIONS);

    // If a new target state is provided (e.g., when moving to a new context), set it.
    if (newTargetState) {
        newTween.set(newTargetState, { duration: NODE_TWEEN_DURATION });
    }

    return {
        id: viewNode.id,
        state: newTween,
        // Deep copy attributes to be safe.
        attributes: viewNode.attributes ? { ...viewNode.attributes } : {},
        status: viewNode.status
    };
}





export function updateNodeLayout(nodeId: NodeId, newX: number, newY: number) {
    const contextId = get(currentContextId);
    const currentCtx = get(contexts).get(contextId);
    if (isNaN(newX) || isNaN(newY)) {
        console.error(`[ContextStore.updateNodeLayout] Received NaN for position! nodeId: ${nodeId}, newX: ${newX}, newY: ${newY}`);
    }
    const viewNode = currentCtx?.viewNodes.get(nodeId); // This is ViewNode { id, state: Tween }
    if (!viewNode) {
        console.warn(`[ContextStore.updateNodeLayout] ViewNode ${nodeId} NOT FOUND in context ${contextId}. Current context view nodes:`, currentCtx?.viewNodes);
    }

    if (viewNode) {
        // 1. Get the current state from the tween
        const currentState = viewNode.state.current;

        // 2. Create the new target state with updated position
        const newState: TweenableNodeState = {
            ...currentState, // Preserve scale, rotation, width, height
            x: newX,
            y: newY
        };

        // 3. Update the tween instantly (no animation during drag)
        viewNode.state.set(newState, { duration: 0 });

        // 4. Trigger reactivity for the contexts store
        // Although we mutated the tween *within* the ViewNode, explicitly
        // signalling that the contexts map has changed ensures Svelte updates.
        contexts.update((map: Map<NodeId, Context>) => map); // Explicitly type map

        // 5. Persistence is now handled by the explicit "Save Layout" button.
        // The old logic for saving on every layout update is removed.
    } else {
        console.warn(`ViewNode ${nodeId} not found in context ${contextId} for layout update`);
    }
}





export async function removeViewNodeFromContext(contextId: NodeId, viewNodeId: NodeId) {
    const currentCtx = get(contexts).get(contextId);

    if (!currentCtx) {
        console.warn(`[removeViewNodeFromContext] Context ${contextId} not found.`);
        return;
    }

    if (!currentCtx.viewNodes.has(viewNodeId)) {
        console.warn(`[removeViewNodeFromContext] ViewNode ${viewNodeId} not found in context ${contextId}.`);
        return;
    }

    // Remove the ViewNode from the context's map
    currentCtx.viewNodes.delete(viewNodeId);

    // Update the contexts store to trigger reactivity
    contexts.update((map: Map<NodeId, Context>) => map);


    // Persist the updated context
    if (activeAdapter) { // Use activeAdapter
        try {
            // Capture current viewport settings before saving
            currentCtx.viewportSettings = { ...viewTransform.current }; // Corrected access
            await activeAdapter.saveContext(currentCtx); // Use activeAdapter
        } catch (error) {
            console.error(`[removeViewNodeFromContext] Error saving context ${contextId} after removing ViewNode ${viewNodeId}:`, error);
        }
    } else {
        console.warn("[removeViewNodeFromContext] activeAdapter not initialized, persistence disabled.");
    }
}





/**
 * Marks a specific ViewNode as modified.
 * This is used to track which nodes need to be saved to the server.
 * @param nodeId The ID of the node to mark as modified.
 */
export function markNodeAsModified(nodeId: NodeId) {
    const contextId = get(currentContextId);
    const allContexts = get(contexts);
    const currentCtx = allContexts.get(contextId);

    if (currentCtx) {
        const viewNode = currentCtx.viewNodes.get(nodeId);
        if (viewNode) {
            if (viewNode.status !== 'modified') {
                viewNode.status = 'modified';
                // Manually trigger reactivity for the contexts store if you want other UI
                // elements to react to the dirty state, e.g., enabling a save button.
                contexts.set(allContexts);
            }
        } else {
            console.warn(`[markNodeAsModified] ViewNode ${nodeId} not found in context ${contextId}.`);
        }
    } else {
        console.warn(`[markNodeAsModified] Current context ${contextId} not found.`);
    }
}





/**
 * Saves the current context's modified nodes to the active persistence layer.
 * After a successful save, it resets the isModified flag on the saved nodes.
 */
export async function saveCurrentContext() {
    const contextId = get(currentContextId);
    const currentCtx = get(contexts).get(contextId);

    if (!currentCtx) {
        console.error('[saveCurrentContext] No current context found to save.');
        return;
    }

    if (!activeAdapter) {
        console.error('[saveCurrentContext] No active persistence adapter found.');
        return;
    }

    try {
        // The adapter's saveContext method is responsible for filtering nodes with status: 'modified'.
        await activeAdapter.saveContext(currentCtx);

        // After a successful save, we don't need to reset the status anymore.
        // The status will be correctly set to 'modified' when the context is loaded next time.
        // This ensures that all nodes from a saved context are considered modifiable.

        console.log(`[saveCurrentContext] Context ${contextId} save operation completed.`);
        // TODO: Add user feedback here (e.g., a toast notification for success)
    } catch (error) {
        console.error(`[saveCurrentContext] Failed to save context ${contextId}:`, error);
        // TODO: Add user feedback for the error
    }
}





/** 
	Triggers a transition to another context.
	Smoothly transitions common nodes, fades out old and fades in new.
*/
export async function switchContext(newContextHandle: NodeHandle, isUndoRedo: boolean = false) {

    // Debug block, has no effect on the function
    try {
        const response = await fetch('http://localhost:7370/api/paths?only_indexed=true');
        if (response.ok) {
            const paths = await response.json();
            apiLogger.log(true, "Indexed paths from server:", paths);
        } else {
            apiLogger.error(true, "Failed to fetch indexed paths:", response.status, response.statusText);
        }
    } catch (error) {
        apiLogger.error(true, "Error fetching indexed paths:", error);
    }

	// --- Phase 1: Resolve NodeHandle to definitive ID and Path ---
	// Regardless of input, we need both values for the rest of the function.
	let newContextId: NodeId;
	let newContextPath: NodePath;

	if (newContextHandle.type === 'uuid') {
		newContextId = newContextHandle.value;
		const localNode = get(nodes).get(newContextId);
		if (localNode && typeof localNode.path === 'string') {
			newContextPath = localNode.path;
		} else {
			const fetchedNode = await activeAdapter.getNode(newContextId);
			if (fetchedNode && typeof fetchedNode.path === 'string') {
				newContextPath = fetchedNode.path;
				nodes.update(n => n.set(fetchedNode.id, fetchedNode)); // Add to store
			} else {
				storeLogger.error(`Could not resolve a path for node ID: ${newContextId}`);
				return;
			}
		}
	} else { // type === 'path'
		newContextPath = newContextHandle.value;
		let foundNode: DataNode | undefined;
		for (const node of get(nodes).values()) {
			if (node.path === newContextPath) {
				foundNode = node;
				break;
			}
		}

		if (foundNode) {
			newContextId = foundNode.id;
		} else {
			const fetchedNode = await activeAdapter.getDataNodeByPath(newContextPath);
			if (fetchedNode) {
				newContextId = fetchedNode.id;
				nodes.update(n => n.set(fetchedNode.id, fetchedNode)); // Add to store
			} else {
				storeLogger.error(`Could not resolve an ID for path: "${newContextPath}"`);
				return;
			}
		}
	}

    const oldContextId = get(currentContextId);
    if (newContextId === oldContextId) return;
    clearSelection();


    if (!isUndoRedo) {
        historyStack.update((stack: NodeId[]) => [...stack, oldContextId]);
        futureStack.set([]);
    }


    if (!activeAdapter) {
        console.error("[switchContext] activeAdapter not available."); return;
    }


    // --- Phase 1: Save Old Context State (Async) ---
    const oldContext = get(contexts).get(oldContextId);

    if (oldContext) {
		
        oldContext.viewportSettings = { ...viewTransform.current };
        storeLogger.log(`Saving old context ${oldContextId} with viewNodes:`, oldContext.viewNodes);

        activeAdapter.saveContext(oldContext)
            .catch(error => console.error(`Error saving old context ${oldContextId}:`, error));

        const targetNodeInOldContext = oldContext.viewNodes.get(newContextId);
        if (targetNodeInOldContext) {
            storeLogger.log(`Target node ${newContextId} (soon to be focal) found in oldContext ${oldContextId}.`);
        } else {
            storeLogger.log(`This means the new focal node is not visible in the old context, so it will be placed at the center of the viewport.`);
        }

    } else {
        storeLogger.warn(`No viewNodes to save for old context ${oldContextId}. This might be expected if the context was never initialized.`);
    }


    // --- Phase 2: Load and Process New Context ---
    try {
        // Determine initial state (x,y,scale,width,height,rotation) for the new focal node based on old context
        const focalInitialState = await _getFocalNodeInitialState(newContextId, oldContext);

        // Load StorableContext via bundle, then convert/merge into in-memory Context with Tweens, add defaults
        let identifierForBundle: string;
        if (activeAdapter instanceof ServerAdapter) {
            const dataNodeForContext = get(nodes).get(newContextId);
            if (!dataNodeForContext || typeof dataNodeForContext.path !== 'string') {
                console.error(`[switchContext] Critical: DataNode or its path not found for newContextId: ${newContextId} when using ServerAdapter.`);
                throw new Error(`DataNode path not found for ${newContextId}, required by ServerAdapter.`);
            }
            identifierForBundle = dataNodeForContext.path;
            // The server endpoint /ctx/root handles the virtual root.
            // An empty string path ("") corresponds to NodePath::root() on the server.
            if (identifierForBundle === "") {
                identifierForBundle = "root";
            }
        } else {
            // LocalAdapter (and potentially other future adapters) might expect the NodeId (UUID)
            identifierForBundle = newContextId;
        }

        const bundle = await activeAdapter.loadContextBundle(identifierForBundle);
        apiLogger.log(`Context Bundle from Server for ${identifierForBundle}:`, bundle);
        const { finalContext: processedContext, wasCreated } = await _loadAndProcessContext(
            newContextId,
            focalInitialState, // Pass the full initial state object
            oldContext, // Pass old context to reuse tweens
            bundle ? bundle.storableContext : undefined // Pass storableContext from bundle
        );


        if (!processedContext) {
            throw new Error("Failed to load or create the new context object.");
        }

        // --- Phase 3: Apply Svelte Stores ---
        // Use nodes and edges directly from the bundle
        const loadedDataNodesMap = new Map<NodeId, DataNode>();
        if (bundle?.nodes) {
            for (const node of bundle.nodes) {
                loadedDataNodesMap.set(node.id, node);
            }
        }

        const loadedEdgesMap = new Map<EdgeId, KartaEdge>();
        if (bundle?.edges) {
            for (const edge of bundle.edges) {
                loadedEdgesMap.set(edge.id, edge);
            }
        }
        // --- DataNode Cleanup ---
        // Before applying the new context, we purge any DataNode from memory that is not
        // part of the incoming context's data bundle. The server is the source of truth.
        // This prevents stale data and ensures virtual nodes are correctly ghosted.
        const incomingDataNodeIds = new Set(loadedDataNodesMap.keys());
        nodes.update(currentNodes => {
            for (const nodeId of currentNodes.keys()) {
                if (!incomingDataNodeIds.has(nodeId)) {
                    currentNodes.delete(nodeId);
                }
            }
            return currentNodes;
        });

        _applyStoresUpdate(newContextId, processedContext, loadedDataNodesMap, loadedEdgesMap);

        // --- Phase 4: Update Viewport ---
        // Only update the viewport if saved settings exist for the context.
        // If processedContext.viewportSettings is undefined (newly created context),
        // the viewport should remain unchanged.
        if (processedContext.viewportSettings !== undefined) {
            viewTransform.set(processedContext.viewportSettings, { duration: VIEWPORT_TWEEN_DURATION });
        }



        try {
            const currentSettings = get(settings);
            if (currentSettings.savelastViewedContextPath) {
                await updateSettings({ lastViewedContextPath: newContextPath });
            }
        } catch (error) {
            console.error('[switchContext] Error saving last context ID to settings:', error);
        }


    } catch (error) {
        console.error(`[switchContext] Error switching context to ${newContextPath}:`, error);
        // Consider reverting to oldContextId or showing an error state
    }
}





async function initializeStores() {
	if (!activeAdapter) {
		console.error("[initializeStores] activeAdapter not initialized. Cannot proceed.");
		// Set a minimal safe state if the adapter is missing.
		nodes.set(new Map());
		edges.set(new Map());
		contexts.set(new Map());
		currentContextId.set(ROOT_NODE_ID);
		viewTransform.set(DEFAULT_VIEWPORT_SETTINGS, { duration: 0 });
		setTool('move');
		return;
	}

    // TODO: Determine how to handle hardcoded contexts
	/*
    if (!USE_SERVER_ADAPTER) {
        try {
            const rootNodeExists = await activeAdapter.getNode(ROOT_NODE_ID);
            if (!rootNodeExists) {
                try {
                    const response = await fetch('/tutorial.json');
                    if (!response.ok) throw new Error(`Failed to fetch: ${response.statusText}`);
                    const tutorialData: KartaExportData = await response.json();
                    if (activeAdapter instanceof LocalAdapter) {
                        await activeAdapter.importData(tutorialData);
                    }
                } catch (importError) {
                    console.error("Failed to import tutorial data:", importError);
                }
            }
        } catch (checkError) {
            console.error("Error checking for root node:", checkError);
        }
    }
    */

	try {
		// Pre-populate the map of all available context paths for the UI.
		const pathsMap = await activeAdapter.getAllContextPaths();
		storeLogger.log('Context paths loaded from adapter:', pathsMap);
		existingContextsMap.set(pathsMap);

		// Determine the initial context to load.
		const currentSettings = get(settings);
		const savedPath = currentSettings.lastViewedContextPath;
		let initialHandle: NodeHandle;
		let fellbackToRoot = false;

		if (currentSettings.savelastViewedContextPath && savedPath) {
			// If a path is saved, we'll try to load it.
			// We optimistically assume the path exists. switchContext will handle errors.
			initialHandle = { type: 'path', value: savedPath };
		} else {
			// If no path is saved or the feature is disabled, default to the root.
			initialHandle = { type: 'uuid', value: ROOT_NODE_ID };
			fellbackToRoot = true;
		}

		// Centralize all loading logic into a single call.
		// switchContext is now responsible for fetching all necessary data.
		await switchContext(initialHandle);

		// Post-load actions.
		if (fellbackToRoot && typeof window !== 'undefined') {
			// Only center the view if we explicitly fell back to the root context.
			// This avoids recentering a context that was loaded with specific viewport settings.
			setTimeout(() => {
				centerOnFocalNode();
			}, 0);
		}

	} catch (error) {
		storeLogger.error("CRITICAL: Initialization failed, setting empty state.", error);
		// Set a minimal safe state on critical failure.
		nodes.set(new Map());
		edges.set(new Map());
		contexts.set(new Map());
		currentContextId.set(ROOT_NODE_ID);
		viewTransform.set(DEFAULT_VIEWPORT_SETTINGS, { duration: 0 });
	}

	// Final setup steps that should run regardless of success or failure.
	if (typeof window !== 'undefined') {
		propertiesPanelPosition.set({ x: window.innerWidth - 320, y: 50 });
	}
	setTool('move');
}

// Export internal helpers AND the initializeStores function
export { _getFocalNodeInitialState, _loadAndProcessContext, _calculateTargetState, _convertStorableViewportSettings, _applyStoresUpdate, initializeStores };