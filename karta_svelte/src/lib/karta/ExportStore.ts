import { writable, derived, get } from 'svelte/store';
import type { NodeId, DataNode } from '$lib/types/types';
import { nodes } from './NodeStore';

export interface ExportableNode {
  id: NodeId;
  node: DataNode;
  addedAt: Date;
  includeChildren: boolean; // For directory nodes
}

/**
 * ExportStore - Manages the collection of nodes selected for export
 * 
 * Users can add individual nodes or entire directories to the export bundle.
 * The store tracks what's been selected and provides utilities for managing the selection.
 */
class ExportStore {
  private _selectedNodes = writable<Map<NodeId, ExportableNode>>(new Map());
  
  // Reactive store of currently selected nodes
  readonly selectedNodes = derived(this._selectedNodes, (nodes) => Array.from(nodes.values()));
  
  // Count of selected nodes
  readonly selectedCount = derived(this._selectedNodes, (nodes) => nodes.size);
  
  // Check if a specific node is selected
  isNodeSelected(nodeId: NodeId): boolean {
    return get(this._selectedNodes).has(nodeId);
  }
  
  /**
   * Add a single node to the export selection
   */
  addNode(nodeId: NodeId, includeChildren: boolean = false): void {
    const allNodes = get(nodes);
    const node = allNodes.get(nodeId);
    
    if (!node) {
      console.warn(`Cannot add node ${nodeId} to export: node not found`);
      return;
    }
    
    this._selectedNodes.update(selected => {
      selected.set(nodeId, {
        id: nodeId,
        node: node,
        addedAt: new Date(),
        includeChildren
      });
      return selected;
    });
    
    console.log(`Added node "${node.attributes.name || nodeId}" to export bundle`);
  }
  
  /**
   * Add a directory and optionally all its children to the export selection
   */
  addDirectory(directoryNodeId: NodeId, recursive: boolean = true): void {
    const allNodes = get(nodes);
    const directoryNode = allNodes.get(directoryNodeId);
    
    if (!directoryNode) {
      console.warn(`Cannot add directory ${directoryNodeId} to export: node not found`);
      return;
    }
    
    // Add the directory node itself
    this.addNode(directoryNodeId, recursive);
    
    if (recursive) {
      // Find all child nodes (nodes whose path starts with this directory's path)
      const directoryPath = directoryNode.path;
      const childNodes = Array.from(allNodes.values()).filter(node => 
        node.path !== directoryPath && node.path.startsWith(directoryPath)
      );
      
      // Add all child nodes
      childNodes.forEach(child => {
        this.addNode(child.id, false); // Don't recursively add children of children
      });
      
      console.log(`Added directory "${directoryNode.attributes.name || directoryNodeId}" and ${childNodes.length} children to export bundle`);
    }
  }
  
  /**
   * Remove a node from the export selection
   */
  removeNode(nodeId: NodeId): void {
    this._selectedNodes.update(selected => {
      selected.delete(nodeId);
      return selected;
    });
  }
  
  /**
   * Clear all selected nodes
   */
  clearSelection(): void {
    this._selectedNodes.set(new Map());
  }
  
  /**
   * Get all node IDs that should be included in the export
   * This includes explicitly selected nodes plus any related contexts/edges
   */
  getExportableNodeIds(): NodeId[] {
    const selected = get(this._selectedNodes);
    return Array.from(selected.keys());
  }
  
  /**
   * Get export statistics
   */
  getExportStats(): {
    nodeCount: number;
    directories: number;
    files: number;
    hasRecursiveDirectories: boolean;
  } {
    const selected = Array.from(get(this._selectedNodes).values());
    
    return {
      nodeCount: selected.length,
      directories: selected.filter(item => item.node.ntype === 'directory').length,
      files: selected.filter(item => item.node.ntype !== 'directory').length,
      hasRecursiveDirectories: selected.some(item => item.includeChildren)
    };
  }
}

// Export singleton instance
export const exportStore = new ExportStore();

// Export actions for use in components
export const exportActions = {
  addNode: exportStore.addNode.bind(exportStore),
  addDirectory: exportStore.addDirectory.bind(exportStore),
  removeNode: exportStore.removeNode.bind(exportStore),
  clearSelection: exportStore.clearSelection.bind(exportStore),
  isNodeSelected: exportStore.isNodeSelected.bind(exportStore),
  getExportableNodeIds: exportStore.getExportableNodeIds.bind(exportStore),
  getExportStats: exportStore.getExportStats.bind(exportStore)
};

// Export reactive stores
export const {
  selectedNodes,
  selectedCount
} = exportStore;
