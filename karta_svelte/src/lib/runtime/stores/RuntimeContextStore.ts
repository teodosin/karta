import { writable, derived, get, type Readable } from 'svelte/store';
import { Tween } from 'svelte/motion';
import type { ViewNode, Context, NodeId, StorableContext, ContextBundle, TweenableNodeState } from '../../types/types';

/**
 * RuntimeContextStore - Read-only store for runtime context management
 * 
 * This store provides a simplified, read-only interface for accessing contexts
 * and view nodes in the runtime environment. It's designed to be loaded with
 * data from exported bundles without editing capabilities.
 */
export class RuntimeContextStore {
    private _contexts = writable<Map<NodeId, Context>>(new Map());

    /**
     * Load contexts from a bundle (typically from an export)
     */
    loadContextsFromBundle(bundle: ContextBundle): void {
        const contextsMap = new Map<NodeId, Context>();
        
        const storableContext = bundle.storableContext;
        
        // Convert StorableContext to Context by recreating Tweens for ViewNodes
        const viewNodesMap = new Map<NodeId, ViewNode>();
        
        storableContext.viewNodes.forEach(([nodeId, storableViewNode]) => {
            const tweenableState: TweenableNodeState = {
                x: storableViewNode.relX,
                y: storableViewNode.relY,
                width: storableViewNode.width,
                height: storableViewNode.height,
                scale: storableViewNode.relScale,
                rotation: storableViewNode.rotation
            };
            
            const viewNode: ViewNode = {
                id: storableViewNode.id,
                state: new Tween<TweenableNodeState>(tweenableState, { duration: 0 }),
                attributes: storableViewNode.attributes,
                status: storableViewNode.status
            };
            viewNodesMap.set(nodeId, viewNode);
        });
        
        const context: Context = {
            id: storableContext.id,
            viewNodes: viewNodesMap,
            viewportSettings: storableContext.viewportSettings ? {
                scale: storableContext.viewportSettings.scale,
                posX: storableContext.viewportSettings.relPosX,
                posY: storableContext.viewportSettings.relPosY
            } : undefined
        };
        
        contextsMap.set(storableContext.id, context);
        this._contexts.set(contextsMap);
    }

    /**
     * Get a specific context by ID (read-only)
     */
    getContext(contextId: NodeId): Context | undefined {
        const contextsMap = get(this._contexts);
        return contextsMap.get(contextId);
    }

    /**
     * Get ViewNodes for a specific context (read-only)
     */
    getViewNodesInContext(contextId: NodeId): Map<NodeId, ViewNode> {
        const context = this.getContext(contextId);
        return context ? new Map(context.viewNodes) : new Map();
    }

    /**
     * Get all contexts as a Map (read-only)
     */
    getAllContexts(): Map<NodeId, Context> {
        return new Map(get(this._contexts));
    }

    /**
     * Check if context exists
     */
    hasContext(contextId: NodeId): boolean {
        return get(this._contexts).has(contextId);
    }

    /**
     * Get all contexts as an array
     */
    getAllContextsArray(): Context[] {
        return Array.from(get(this._contexts).values());
    }

    /**
     * Get context count
     */
    getContextCount(): number {
        return get(this._contexts).size;
    }

    /**
     * Clear all contexts
     */
    clear(): void {
        this._contexts.set(new Map());
    }

    /**
     * Get reactive store for all contexts (read-only)
     */
    get contexts(): Readable<Map<NodeId, Context>> {
        return derived(this._contexts, $contexts => new Map($contexts));
    }
}

// Export singleton instance for runtime use
export const runtimeContextStore = new RuntimeContextStore();
