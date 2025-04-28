import { writable, get, derived } from 'svelte/store';
import { Tween } from 'svelte/motion';
import { cubicOut } from 'svelte/easing';
import { localAdapter } from '../util/LocalAdapter';
import type { DataNode, KartaEdge, ViewNode, Context, NodeId, EdgeId, AbsoluteTransform, ViewportSettings, TweenableNodeState, StorableContext, StorableViewNode, StorableViewportSettings, Tool } from '../types/types'; // Added EdgeId and Tool
import { getDefaultViewNodeStateForType } from '$lib/node_types/registry';
import { nodes, _ensureDataNodeExists } from './NodeStore'; // Assuming NodeStore exports these
import { edges } from './EdgeStore'; // Assuming EdgeStore exports this
import { viewTransform, DEFAULT_VIEWPORT_SETTINGS, VIEWPORT_TWEEN_DURATION, DEFAULT_FOCAL_TRANSFORM } from './ViewportStore'; // Assuming ViewportStore exports these
import { historyStack, futureStack } from './HistoryStore'; // Assuming HistoryStore exports these
import { clearSelection } from './SelectionStore'; // Assuming SelectionStore exports these
import { propertiesPanelPosition, setPropertiesPanelNode, setPropertiesPanelVisibility } from './UIStateStore'; // Assuming UIStateStore exports these
import { setTool, currentTool, initializeTools } from './ToolStore'; // Assuming ToolStore exports these and initializeTools
import { settings } from './SettingsStore'; // Import settings store


// Define the Root Node ID
export const ROOT_NODE_ID = '00000000-0000-0000-0000-000000000000';
const LAST_CONTEXT_STORAGE_KEY = 'kartaLastContextId'; // Key for localStorage

// Constants
const NODE_TWEEN_DURATION = 250; // Duration for node transitions
const NODE_TWEEN_OPTIONS = { duration: NODE_TWEEN_DURATION, easing: cubicOut };

// Stores
export const contexts = writable<Map<NodeId, Context>>(new Map());
export const currentContextId = writable<NodeId>(ROOT_NODE_ID);

// Derived Store for Current Context's ViewNodes
export const currentViewNodes = derived(
	[currentContextId, contexts],
	([$currentContextId, $contexts]) => {
		return $contexts.get($currentContextId)?.viewNodes ?? new Map<NodeId, ViewNode>();
	}
);

// Internal Helper Functions
async function _getFocalNodeInitialState(targetNodeId: NodeId, oldContext: Context | undefined): Promise<TweenableNodeState> {
    const targetViewNodeInOldCtx = oldContext?.viewNodes.get(targetNodeId);

    if (targetViewNodeInOldCtx) {
        // Use existing state from the previous context
        return { ...targetViewNodeInOldCtx.state.current }; // Return a copy
    } else {
        // Node not visible in old context, calculate defaults
        console.log(`[_getFocalNodeInitialState] Node ${targetNodeId} not found in old context. Calculating defaults.`);
        const dataNode = await _ensureDataNodeExists(targetNodeId); // Ensure DataNode exists to get ntype
        const defaultState: { width: number; height: number; scale: number; rotation: number; } = dataNode ? getDefaultViewNodeStateForType(dataNode.ntype) : getDefaultViewNodeStateForType('generic'); // Fallback to generic

        // Combine default placement with default dimensions/rotation
        return {
            ...DEFAULT_FOCAL_TRANSFORM, // Default x, y, scale
            width: defaultState.width,
            height: defaultState.height,
            rotation: defaultState.rotation
        };
    }
}

async function _loadAndProcessContext(
    contextId: NodeId,
    focalInitialStateFromOldContext: TweenableNodeState, // Full initial state including dimensions
    oldContext: Context | undefined // Previous context for potential tween reuse
): Promise<{ finalContext: Context | undefined, wasCreated: boolean }> {

    if (!localAdapter) return { finalContext: undefined, wasCreated: false };

    const storableContext = await localAdapter.getContext(contextId);
    let contextWasCreated = false;
    const finalViewNodes = new Map<NodeId, ViewNode>();
    let finalViewportSettings: ViewportSettings | undefined = undefined;

    if (storableContext) {
        // --- Context exists in DB ---
        console.log(`[_loadAndProcessContext] Context ${contextId} loaded from DB.`);
        // Convert StorableViewNodes to ViewNodes with Tweens, reusing old tweens if possible
        for (const [nodeId, storableNode] of storableContext.viewNodes) {
            // Extract only the placement part needed by _calculateTargetState
            const focalPlacement: AbsoluteTransform = { x: focalInitialStateFromOldContext.x, y: focalInitialStateFromOldContext.y, scale: focalInitialStateFromOldContext.scale };
            const targetState = _calculateTargetState(nodeId, contextId, focalPlacement, storableNode);
            const existingViewNode = oldContext?.viewNodes.get(nodeId);

            if (existingViewNode) {
                existingViewNode.state.set(targetState, NODE_TWEEN_OPTIONS); // Update existing tween
                existingViewNode.attributes = storableNode.attributes; // Copy attributes
                finalViewNodes.set(nodeId, existingViewNode); // Reuse ViewNode object
            } else {
                // Create new ViewNode with a new Tween, starting from target state? Or current if exists? Start from target.
                finalViewNodes.set(nodeId, { id: nodeId, state: new Tween(targetState, NODE_TWEEN_OPTIONS), attributes: storableNode.attributes });
            }
        }
        // Convert stored viewport settings
        finalViewportSettings = _convertStorableViewportSettings(storableContext.viewportSettings, finalViewNodes.get(contextId));

    } else {
        // --- Context needs creation ---
        console.log(`[_loadAndProcessContext] Context ${contextId} not found in DB, creating new.`);
        contextWasCreated = true;
        const focalDataNode = await localAdapter.getNode(contextId);
        if (!focalDataNode) throw new Error(`Cannot create context for non-existent node: ${contextId}`);
        if (contextId === ROOT_NODE_ID) {
             console.log(`[_loadAndProcessContext] Root context creation: Loaded focalDataNode ntype: ${focalDataNode.ntype}`);
        }

        // Create initial state and ViewNode for the focal node
        // Use type-specific defaults for size/scale/rotation
        if (contextId === ROOT_NODE_ID) {
             console.log(`[_loadAndProcessContext] Root context creation: Getting default view state for ntype: ${focalDataNode.ntype}`);
        }
        // Use the state passed in, which already contains defaults or state from old context
        const correctedFocalInitialState: TweenableNodeState = {
             x: focalInitialStateFromOldContext.x,
             y: focalInitialStateFromOldContext.y,
             scale: focalInitialStateFromOldContext.scale,
             rotation: focalInitialStateFromOldContext.rotation,
             width: focalInitialStateFromOldContext.width, // Use width from old context/defaults
             height: focalInitialStateFromOldContext.height // Use height from old context/defaults
        };
        finalViewNodes.set(contextId, { id: contextId, state: new Tween(correctedFocalInitialState, { duration: 0 }) });
        // For newly created contexts, don't set viewport settings yet.
        // Let the viewport remain where it is. Settings will be saved on first interaction/switch away.
        // For newly created contexts, don't set viewport settings here.
        // Let finalViewportSettings remain undefined.
    }

     // --- Add Previous Focal Node (if context is new and applicable) ---
    	if (contextWasCreated) {
    		const currentHistory: NodeId[] = get(historyStack); // Explicitly type
    		const previousContextId = currentHistory.length > 0 ? currentHistory[currentHistory.length - 1] : null;
    		if (previousContextId && previousContextId !== contextId && !finalViewNodes.has(previousContextId)) {
    			const previousFocalViewNode = oldContext?.viewNodes.get(previousContextId);
    			if (previousFocalViewNode) {
    				finalViewNodes.set(previousContextId, previousFocalViewNode);
    			}
    		}
    	}

    // --- Add Default Connected Nodes (if context didn't just load them) ---
    // This ensures nodes connected *after* the context was last saved are still added by default
    const directlyConnectedEdges = await localAdapter.getEdgesByNodeIds([contextId]);
    const connectedNodeIdsToAdd = new Set<NodeId>();
    for (const edge of directlyConnectedEdges.values()) {
        const neighborId = edge.source === contextId ? edge.target : edge.source;
        if (neighborId !== contextId && !finalViewNodes.has(neighborId)) {
            connectedNodeIdsToAdd.add(neighborId);
        }
    }

    if (connectedNodeIdsToAdd.size > 0) {
        console.log(`[_loadAndProcessContext] Adding ${connectedNodeIdsToAdd.size} default connected nodes.`);
        contextWasCreated = true; // Mark modified if defaults added
        let defaultOffset = 150;
        const angleIncrement = 360 / connectedNodeIdsToAdd.size;
        let currentAngle = 0;
        const focalState = finalViewNodes.get(contextId)!.state.current; // Focal state must exist

        for (const connectedId of connectedNodeIdsToAdd) {
            const existingViewNodeInOldContext = oldContext?.viewNodes.get(connectedId);
            // Calculate default position relative to current focal state
            const angleRad = (currentAngle * Math.PI) / 180;
            const defaultRelX = defaultOffset * Math.cos(angleRad);
            const defaultRelY = defaultOffset * Math.sin(angleRad);
            const scaledRelX = defaultRelX * focalState.scale;
            const scaledRelY = defaultRelY * focalState.scale;
            const defaultAbsX = focalState.x + scaledRelX;
            const defaultAbsY = focalState.y + scaledRelY;
            const defaultState: TweenableNodeState = {
                x: defaultAbsX, y: defaultAbsY, scale: focalState.scale, rotation: 0,
                width: 100, height: 100 // Default size - TODO: Get from DataNode?
            };

            if (existingViewNodeInOldContext) {
                finalViewNodes.set(connectedId, existingViewNodeInOldContext);
            } else {
                // Create new ViewNode/Tween
                finalViewNodes.set(connectedId, { id: connectedId, state: new Tween(defaultState, NODE_TWEEN_OPTIONS) });
            }
            currentAngle += angleIncrement;
        }
    }

    // Construct the final Context object
    const finalContext: Context = {
        id: contextId,
        viewNodes: finalViewNodes,
        viewportSettings: finalViewportSettings // Allow undefined if context was new/had no settings
    };

    // Save immediately if it was newly created or defaults were added
    if (contextWasCreated && localAdapter) {
        await localAdapter.saveContext(finalContext);
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
            x: focalPlacement.x, y: focalPlacement.y,
            scale: focalPlacement.scale, rotation: storableNode.rotation,
            width: storableNode.width, height: storableNode.height
        };
    } else {
        const absScale = focalPlacement.scale * storableNode.relScale;
        const scaledRelX = storableNode.relX * focalPlacement.scale;
        const scaledRelY = storableNode.relY * focalPlacement.scale;
        const absX = focalPlacement.x + scaledRelX;
        const absY = focalPlacement.y + scaledRelY;
        return {
            x: absX, y: absY, scale: absScale, rotation: storableNode.rotation,
            width: storableNode.width, height: storableNode.height
        };
    }
}

function _convertStorableViewportSettings(
    storableSettings: StorableViewportSettings | undefined,
    focalViewNode: ViewNode | undefined
): ViewportSettings | undefined {
    if (!storableSettings) return undefined;

    const focalState = focalViewNode?.state.current;
    if (focalState) {
        const absPosX = storableSettings.relPosX - (focalState.x * storableSettings.scale);
        const absPosY = storableSettings.relPosY - (focalState.y * storableSettings.scale);
        return { scale: storableSettings.scale, posX: absPosX, posY: absPosY };
    } else {
        console.warn("Focal node state not found when converting viewport settings.");
        return { ...DEFAULT_VIEWPORT_SETTINGS };
    }
}

async function _loadContextData(context: Context | undefined): Promise<{ loadedDataNodes: Map<NodeId, DataNode>, loadedEdges: Map<EdgeId, KartaEdge> }> {
    let loadedDataNodes = new Map<NodeId, DataNode>();
    let loadedEdges = new Map<EdgeId, KartaEdge>();

    if (!localAdapter || !context) {
        return { loadedDataNodes, loadedEdges };
    }

    const viewNodeIds = Array.from(context.viewNodes.keys());
    if (viewNodeIds.length > 0) {
        loadedDataNodes = await localAdapter.getDataNodesByIds(viewNodeIds);

        loadedEdges = await localAdapter.getEdgesByNodeIds(viewNodeIds);
    } else {
        console.warn(`[_loadContextData] Context ${context.id} has no view nodes.`);
    }
    return { loadedDataNodes, loadedEdges };
}

function _applyStoresUpdate(
    newContextId: NodeId,
    processedContext: Context, // The fully processed context with ViewNodes/Tweens
    loadedDataNodesForContext: Map<NodeId, DataNode>, // Renamed for clarity
    loadedEdgesForContext: Map<EdgeId, KartaEdge> // Renamed for clarity
) {
    // --- Update nodes store (Merge) ---
    // Keep all existing nodes, only add/update the ones loaded for the new context
    nodes.update(currentNodesMap => {
        const nextNodes = new Map<NodeId, DataNode>(currentNodesMap); // Start with all existing nodes
        for (const [nodeId, dataNode] of loadedDataNodesForContext.entries()) {
            nextNodes.set(nodeId, dataNode); // Add or overwrite node data loaded for this context
        }
        console.log(`[_applyStoresUpdate] Merged ${loadedDataNodesForContext.size} nodes into global $nodes store.`);
        console.log(`[_applyStoresUpdate] Final $nodes map size: ${nextNodes.size}`);
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
    console.log(`[_applyStoresUpdate] Set $edges store with ${nextEdges.size} edges relevant to context ${newContextId}.`);

    // --- Update contexts store ---
    // Set the fully processed context (containing ViewNodes with Tweens)
    contexts.update((map: Map<NodeId, Context>) => map.set(newContextId, processedContext)); // Explicitly type map

    // --- Update current context ID ---
    currentContextId.set(newContextId);
   }


export function updateNodeLayout(nodeId: NodeId, newX: number, newY: number) {
    const contextId = get(currentContextId);
    const currentCtx = get(contexts).get(contextId);
    const viewNode = currentCtx?.viewNodes.get(nodeId); // This is ViewNode { id, state: Tween }

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

        // 5. Persist the change asynchronously
        // saveContext reads the .current state from the tween when converting
        if (localAdapter && currentCtx) {
            // Debounce saving? For now, save on every update during drag.
            currentCtx.viewportSettings = { ...viewTransform.current }; // Capture viewport - Access .current directly
            localAdapter.saveContext(currentCtx)
                .catch(err => console.error(`Error saving context ${contextId} during layout update:`, err));
        }
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

    console.log(`[removeViewNodeFromContext] Removed ViewNode ${viewNodeId} from context ${contextId}.`);

    // Persist the updated context
    if (localAdapter) {
        try {
            // Capture current viewport settings before saving
            currentCtx.viewportSettings = { ...viewTransform.current }; // Corrected access
            await localAdapter.saveContext(currentCtx);
            console.log(`[removeViewNodeFromContext] Saved context ${contextId} after removing ViewNode ${viewNodeId}.`);
        } catch (error) {
            console.error(`[removeViewNodeFromContext] Error saving context ${contextId} after removing ViewNode ${viewNodeId}:`, error);
        }
    } else {
        console.warn("[removeViewNodeFromContext] LocalAdapter not initialized, persistence disabled.");
    }
}

export async function switchContext(newContextId: NodeId, isUndoRedo: boolean = false) { // Added isUndoRedo flag
	const oldContextId = get(currentContextId);
	if (newContextId === oldContextId) return; // No change

	clearSelection(); // Clear selection when switching context

	// --- History Management ---
    if (!isUndoRedo) {
        historyStack.update((stack: NodeId[]) => [...stack, oldContextId]); // Explicitly type stack
        futureStack.set([]); // Clear future stack on new action
        console.log(`[switchContext] Pushed ${oldContextId} to history.`);
    }
    // --- End History Management ---


    if (!localAdapter) {
        console.error("[switchContext] LocalAdapter not available."); return;
    }
    console.log(`[switchContext] Switching from ${oldContextId} to ${newContextId}`);

    // --- Phase 1: Save Old Context State (Async) ---
    const oldContext = get(contexts).get(oldContextId);
    if (oldContext) {
        oldContext.viewportSettings = { ...viewTransform.current }; // Capture current viewport - Access .current directly
        localAdapter.saveContext(oldContext) // Adapter converts ViewNode with Tween back to Storable
            .then(() => console.log(`[switchContext] Old context ${oldContextId} saved.`))
            .catch(error => console.error(`[switchContext] Error saving old context ${oldContextId}:`, error));
    } else {
        console.warn(`[switchContext] Old context ${oldContextId} not found in memory for saving.`);
    }

    // --- Phase 2: Load and Process New Context ---
    try {
        // Determine initial state (x,y,scale,width,height,rotation) for the new focal node based on old context
        const focalInitialState = await _getFocalNodeInitialState(newContextId, oldContext);

        // Load StorableContext, convert/merge into in-memory Context with Tweens, add defaults
        const { finalContext: processedContext } = await _loadAndProcessContext(
            newContextId,
            focalInitialState, // Pass the full initial state object
            oldContext // Pass old context to reuse tweens
        );

        if (!processedContext) {
            throw new Error("Failed to load or create the new context object.");
        }

        // Load necessary DataNodes and Edges for the nodes now in the context
        const loadedData = await _loadContextData(processedContext);
        const loadedDataNodes = loadedData.loadedDataNodes;
        const loadedEdges = loadedData.loadedEdges;

        // --- Phase 3: Apply Svelte Stores ---
        _applyStoresUpdate(newContextId, processedContext, loadedDataNodes, loadedEdges);

        // --- Phase 4: Update Viewport ---
        // --- Phase 4: Update Viewport ---
        // Only update the viewport if saved settings exist for the context.
        // If processedContext.viewportSettings is undefined (newly created context),
        // the viewport should remain unchanged.
        if (processedContext.viewportSettings !== undefined) {
            viewTransform.set(processedContext.viewportSettings, { duration: VIEWPORT_TWEEN_DURATION }); // Restore tween duration
            console.log(`[switchContext] Loaded viewport settings for context ${newContextId}. Applying with tween.`);
        } else {
            console.log(`[switchContext] Context ${newContextId} was newly created or had no saved viewport settings. Viewport position maintained.`);
        }

  console.log(`[switchContext] Successfully switched to context ${newContextId}`);

  // --- Save Last Context ID ---
  try {
   const currentSettings = get(settings);
   if (currentSettings.saveLastViewedContext && typeof window !== 'undefined' && window.localStorage) {
    localStorage.setItem(LAST_CONTEXT_STORAGE_KEY, newContextId);
    console.log(`[switchContext] Saved last context ID ${newContextId} to localStorage.`);
   }
  } catch (error) {
   console.error('[switchContext] Error saving last context ID to localStorage:', error);
  }
  // --- End Save Last Context ID ---

 } catch (error) {
  console.error(`[switchContext] Error switching context to ${newContextId}:`, error);
        // Consider reverting to oldContextId or showing an error state
    }
}

async function initializeStores() { // Remove export keyword here
	console.log('[initializeStores] Initializing Karta stores...');
	initializeTools(); // Initialize tool instances here

	// Ensure currentTool is not null before calling activate (this check might be redundant after initializeTools)
    const currentToolInstance = get(currentTool);
    if (currentToolInstance) {
        currentToolInstance.activate(); // Activate default tool
    }


    if (!localAdapter) {
        console.error("[initializeStores] LocalAdapter not initialized. Cannot proceed.");
        // Set default empty state on critical error
        nodes.set(new Map());
        edges.set(new Map());
        contexts.set(new Map());
        currentContextId.set(ROOT_NODE_ID); // Default to root ID even on error
        viewTransform.set(DEFAULT_VIEWPORT_SETTINGS, { duration: 0 });
        setTool('move');
        return;
    }

    try {
        // 0. Initialize stores to empty state
        console.log('[initializeStores] Initializing stores to empty state.');
        nodes.set(new Map());
        edges.set(new Map());
        contexts.set(new Map());
        currentContextId.set(ROOT_NODE_ID); // Temporarily set to root

        // 1. Determine Target Initial Context ID based on settings
        let targetInitialContextId = ROOT_NODE_ID; // Default
        try {
            const currentSettings = get(settings); // Read settings (should be loaded by +layout.svelte)
            if (currentSettings.saveLastViewedContext && typeof window !== 'undefined' && window.localStorage) {
                const savedId = localStorage.getItem(LAST_CONTEXT_STORAGE_KEY);
                if (savedId) {
                    // Check if the saved ID corresponds to an existing node in the DB
                    const nodeExists = await localAdapter.getNode(savedId);
                    if (nodeExists) {
                        targetInitialContextId = savedId;
                        console.log(`[initializeStores] Found and validated last context ID in localStorage: ${targetInitialContextId}`);
                    } else {
                        console.warn(`[initializeStores] Saved last context ID ${savedId} not found in DB. Defaulting to ROOT.`);
                        targetInitialContextId = ROOT_NODE_ID;
                    }
                } else {
                    console.log(`[initializeStores] No last context ID found, defaulting to ROOT: ${ROOT_NODE_ID}`);
                    targetInitialContextId = ROOT_NODE_ID;
                }
            } else {
                console.log(`[initializeStores] Setting disabled or localStorage not available, defaulting to ROOT: ${ROOT_NODE_ID}`);
                targetInitialContextId = ROOT_NODE_ID;
            }
        } catch (error) {
            console.error('[initializeStores] Error reading settings or localStorage for last context ID:', error);
            targetInitialContextId = ROOT_NODE_ID; // Fallback to ROOT on error
        }

        // 2. Ensure and Load Initial DataNode (Target or Root)
        let initialDataNode = await localAdapter.getNode(targetInitialContextId);

        if (!initialDataNode) {
            console.warn(`[initializeStores] Target node ${targetInitialContextId} not found in DB. Falling back to ROOT.`);
            targetInitialContextId = ROOT_NODE_ID; // Reset target ID to root
            initialDataNode = await localAdapter.getNode(ROOT_NODE_ID);
            if (!initialDataNode) {
                // If even the root is not in the DB, ensure it exists
                initialDataNode = await _ensureDataNodeExists(ROOT_NODE_ID); // This creates and saves if needed
                if (!initialDataNode) {
                    throw new Error("CRITICAL: Root DataNode could not be found or created during initialization.");
                }
            }
        }

        // Add the successfully loaded/ensured initial node to the nodes store
        nodes.update(n => n.set(initialDataNode!.id, initialDataNode!));
        console.log(`[initializeStores] Initial node ${initialDataNode.id} loaded into store.`);


        // 3. Ensure Root Node has isSystemNode flag (if the initial node IS the root node)
        if (initialDataNode.id === ROOT_NODE_ID && !initialDataNode.attributes?.isSystemNode) {
            console.warn(`[initializeStores] Root node ${ROOT_NODE_ID} is missing the isSystemNode flag. Adding and saving...`);
            initialDataNode.attributes = { ...initialDataNode.attributes, isSystemNode: true };
            initialDataNode.modifiedAt = Date.now();
            try {
                await localAdapter.saveNode(initialDataNode);
                console.log(`[initializeStores] Successfully added isSystemNode flag to root node.`);
                // Update the node in the store as well
                nodes.update(n => n.set(initialDataNode!.id, initialDataNode!));
            } catch (saveError) {
                console.error(`[initializeStores] CRITICAL: Failed to save isSystemNode flag to root node:`, saveError);
                // Continue initialization, but the root might remain unprotected if saving failed.
            }
        }


        // 4. Load Initial Context & Data (Generalized)
        // Construct the default initial state for the root node
        const rootDefaultViewState = getDefaultViewNodeStateForType('root');
        const rootInitialState: TweenableNodeState = {
            ...DEFAULT_FOCAL_TRANSFORM, // x, y, scale
            width: rootDefaultViewState.width,
            height: rootDefaultViewState.height,
            rotation: rootDefaultViewState.rotation
        };

        const { finalContext } = await _loadAndProcessContext(
            initialDataNode.id, // Use the validated ID
            rootInitialState, // Pass the full default state
            undefined // No old context on init
        );
        let initialProcessedContext = finalContext;

        if (!initialProcessedContext) {
            throw new Error(`Failed to load or process initial context for node: ${initialDataNode.id}`);
        }

        // Load necessary DataNodes and Edges for the nodes now in the context
        // Note: loadedDataNodesForContext here will only contain nodes *in the initial context*
        const loadedData = await _loadContextData(initialProcessedContext);
        const loadedDataNodesForContext = loadedData.loadedDataNodes; // Renamed for clarity
        const loadedEdges = loadedData.loadedEdges;

        // 5. Apply Initial State
        // _applyStoresUpdate will now merge loadedDataNodesForContext into the existing $nodes store
        _applyStoresUpdate(initialDataNode.id, initialProcessedContext, loadedDataNodesForContext, loadedEdges);

        // 6. Set Initial Viewport
        // If loading the root context and no last context was saved, center the root node.
        // Otherwise, use loaded viewport settings or defaults.
        let initialViewportSettings = initialProcessedContext.viewportSettings || DEFAULT_VIEWPORT_SETTINGS;

        if (initialDataNode.id === ROOT_NODE_ID && targetInitialContextId === ROOT_NODE_ID && typeof window !== 'undefined') {
            // Calculate translation to center the root node (at 0,0)
            const centerX = window.innerWidth / 2;
            const centerY = window.innerHeight / 2;
            // The viewport position is the top-left corner in canvas coordinates.
            // To center the node at (0,0), the top-left should be at (centerX / scale, centerY / scale)
            // However, viewTransform stores the top-left in screen coordinates relative to the canvas origin.
            // So, the required translation in screen space is simply (centerX, centerY).
            // The viewTransform store's posX/posY represent the canvas origin's screen coordinates.
            // To move the canvas origin so that the root node (at canvas 0,0) is at screen (centerX, centerY),
            // the canvas origin needs to be at screen (centerX, centerY).
            // The viewTransform posX/posY are the screen coordinates of the canvas origin.
            // So, we want viewTransform.posX = centerX and viewTransform.posY = centerY.
            // The scale should be the default scale.
            initialViewportSettings = {
                scale: DEFAULT_VIEWPORT_SETTINGS.scale, // Use default scale
                posX: centerX,
                posY: centerY
            };
            console.log(`[initializeStores] Centering root node at screen (${centerX}, ${centerY}).`);
        } else {
            console.log(`[initializeStores] Loading context ${initialDataNode.id}. Using saved or default viewport settings.`);
        }

        viewTransform.set(initialViewportSettings, { duration: 0 }); // Set instantly

        console.log(`[initializeStores] Stores initialized successfully, starting context: ${initialDataNode.id}`);

    } catch (error) {
        console.error("[initializeStores] Error during store initialization:", error);
        // Set default empty state on error
        nodes.set(new Map());
        edges.set(new Map());
        contexts.set(new Map());
        currentContextId.set(ROOT_NODE_ID); // Default to root ID even on error
        viewTransform.set(DEFAULT_VIEWPORT_SETTINGS, { duration: 0 });
    }

    // 7. Set Initial Properties Panel Position (calculated only in browser)
    if (typeof window !== 'undefined') {
        propertiesPanelPosition.set({ x: window.innerWidth - 320, y: 50 }); // Set browser-dependent default
    }


    // 8. Set Initial Tool (runs even if init fails partially)
    setTool('move');
}

// Export internal helpers AND the initializeStores function
export { _getFocalNodeInitialState, _loadAndProcessContext, _calculateTargetState, _convertStorableViewportSettings, _loadContextData, _applyStoresUpdate, initializeStores };