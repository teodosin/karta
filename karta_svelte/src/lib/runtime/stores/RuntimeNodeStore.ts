import { writable, derived, get, type Readable } from 'svelte/store';
import type { DataNode, NodeId } from '../../types/types';

/**
 * RuntimeNodeStore - Read-only store for runtime node rendering
 * 
 * This store provides a simplified, read-only interface for accessing nodes
 * in the runtime environment. It's designed to be loaded with data from
 * exported bundles or server responses without editing capabilities.
 */
export class RuntimeNodeStore {
    private _nodes = writable<Map<NodeId, DataNode>>(new Map());
    
    // Read-only reactive store
    public readonly nodes: Readable<Map<NodeId, DataNode>> = derived(
        this._nodes,
        ($nodes) => new Map($nodes) // Return immutable copy
    );

    /**
     * Load nodes from bundle data
     */
    loadNodes(nodeData: DataNode[]): void {
        const nodeMap = new Map<NodeId, DataNode>();
        
        for (const node of nodeData) {
            nodeMap.set(node.id, { ...node }); // Defensive copy
        }
        
        this._nodes.set(nodeMap);
    }

    /**
     * Get a specific node by ID (read-only)
     */
    getNode(nodeId: NodeId): DataNode | undefined {
        const nodesMap = get(this._nodes);
        return nodesMap.get(nodeId);
    }

    /**
     * Get all nodes as a Map (read-only)
     */
    getNodesMap(): Map<NodeId, DataNode> {
        return new Map(get(this._nodes));
    }

    /**
     * Check if a node exists
     */
    hasNode(nodeId: NodeId): boolean {
        return get(this._nodes).has(nodeId);
    }

    /**
     * Get all nodes as an array
     */
    getAllNodes(): DataNode[] {
        return Array.from(get(this._nodes).values());
    }

    /**
     * Clear all nodes
     */
    clear(): void {
        this._nodes.set(new Map());
    }

    /**
     * Get node count
     */
    getNodeCount(): number {
        return get(this._nodes).size;
    }
}

// Export singleton instance for runtime use
export const runtimeNodeStore = new RuntimeNodeStore();
