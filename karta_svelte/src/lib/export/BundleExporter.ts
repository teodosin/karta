import { get } from 'svelte/store';
import type { DataNode, KartaEdge } from '../types/types';
import { nodes } from '../karta/NodeStore';
import { contexts } from '../karta/ContextStore';
import { edges } from '../karta/EdgeStore';

// Bundle types (will eventually move to @karta/core)
export interface BundleEdge {
  id: string;
  from: string;
  to: string;
  edgeType: string;
  attributes: Record<string, unknown>;
}

export interface KartaBundle {
  version: number;
  exportedAt: string;
  metadata?: {
    title?: string;
    description?: string;
    author?: string;
    minRuntimeVersion?: string;
  };
  nodes: DataNode[];
  contexts: Record<string, { viewNodes: Record<string, any> }>;
  edges: BundleEdge[];
  assets?: Record<string, unknown>;
}

/**
 * BundleExporter - Creates exportable bundles from current editor state
 * 
 * This class reads from existing editor stores without modifying them,
 * creating MIT-licensed bundles that can be loaded by @karta/runtime.
 */
export class BundleExporter {
  /**
   * Export the current editor state as a KartaBundle
   * Safe to call - only reads from stores, never modifies them
   */
  static async exportCurrentState(options: {
    title?: string;
    description?: string;
    author?: string;
    includeAssets?: boolean;
  } = {}): Promise<KartaBundle> {
    
    // Read current state from editor stores (safe read-only operations)
    const currentNodes = get(nodes);
    const currentContexts = get(contexts);
    const currentEdges = get(edges);

    // Convert Map<NodeId, DataNode> to DataNode[]
    const bundleNodes: DataNode[] = Array.from(currentNodes.values());
    
    // Convert edges to bundle format
    const bundleEdges: BundleEdge[] = Array.from(currentEdges.values()).map(edge => ({
      id: edge.id,
      from: edge.source, // KartaEdge uses 'source' 
      to: edge.target,   // KartaEdge uses 'target'
      edgeType: 'default', // KartaEdge doesn't have edgeType, use default
      attributes: edge.attributes || {}
    }));

    // Convert contexts to bundle format
    const bundleContexts: Record<string, { viewNodes: Record<string, any> }> = {};
    for (const [contextId, context] of currentContexts) {
      bundleContexts[contextId] = {
        viewNodes: context.viewNodes ? Object.fromEntries(context.viewNodes) : {}
      };
    }

    const bundle: KartaBundle = {
      version: 1,
      exportedAt: new Date().toISOString(),
      metadata: {
        title: options.title || 'Exported Karta Network',
        description: options.description,
        author: options.author,
        minRuntimeVersion: '0.1.0'
      },
      nodes: bundleNodes,
      contexts: bundleContexts,
      edges: bundleEdges
    };

    // TODO: Add asset handling when requested
    if (options.includeAssets) {
      bundle.assets = await this.exportAssets();
    }

    return bundle;
  }

  /**
   * Export bundle as JSON string
   */
  static async exportAsJsonString(options?: Parameters<typeof BundleExporter.exportCurrentState>[0]): Promise<string> {
    const bundle = await this.exportCurrentState(options);
    return JSON.stringify(bundle, null, 2);
  }

  /**
   * Download bundle as a file
   */
  static async downloadBundle(filename?: string, options?: Parameters<typeof BundleExporter.exportCurrentState>[0]): Promise<void> {
    const jsonString = await this.exportAsJsonString(options);
    const blob = new Blob([jsonString], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    
    const a = document.createElement('a');
    a.href = url;
    a.download = filename || `karta-export-${new Date().toISOString().split('T')[0]}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  /**
   * Export assets (placeholder for future implementation)
   */
  private static async exportAssets(): Promise<Record<string, unknown>> {
    // TODO: Implement asset export
    // This would handle images, files, etc.
    return {};
  }

  /**
   * Validate a bundle against the expected schema
   */
  static validateBundle(bundle: unknown): bundle is KartaBundle {
    if (!bundle || typeof bundle !== 'object') return false;
    
    const b = bundle as Partial<KartaBundle>;
    
    return (
      typeof b.version === 'number' &&
      typeof b.exportedAt === 'string' &&
      Array.isArray(b.nodes) &&
      Array.isArray(b.edges) &&
      typeof b.contexts === 'object' &&
      b.contexts !== null
    );
  }
}
