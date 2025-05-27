import type {
    DataNode,
    KartaEdge,
    NodeId,
    StorableContext,
    AssetData,
    Context, // For saveContext, though it will be stubbed
} from '../types/types';
import type { PersistenceService } from './PersistenceService';

const SERVER_BASE_URL = 'http://localhost:7370'; // As determined

export class ServerAdapter implements PersistenceService {
    constructor() {
        // Initialization if needed in the future
        console.log('[ServerAdapter] Initialized');
    }

    /**
     * Fetches the complete bundle of data required to render a context from the server.
     * This includes DataNodes, KartaEdges, and the StorableContext (layout).
     * @param contextPath The relative path of the context from the vault root.
     * @returns A promise that resolves with an object containing nodes, edges, and storableContext, or undefined on error.
     */
    async loadContextBundleByPath(contextPath: string): Promise<{ nodes: DataNode[], edges: KartaEdge[], storableContext: StorableContext } | undefined> {
        const encodedPath = encodeURIComponent(contextPath);
        const url = `${SERVER_BASE_URL}/ctx/${encodedPath}`;

        try {
            console.log(`[ServerAdapter] Fetching context bundle from: ${url}`);
            const response = await fetch(url);

            if (!response.ok) {
                console.error(`[ServerAdapter] Error fetching context bundle for path "${contextPath}". Status: ${response.status} ${response.statusText}`);
                const errorBody = await response.text();
                console.error(`[ServerAdapter] Error body: ${errorBody}`);
                return undefined;
            }

            const data = await response.json();

            // Expected server response: [DataNode[], KartaEdge[], StorableContextData]
            if (Array.isArray(data) && data.length === 3) {
                const serverNodes = data[0] as DataNode[];
                const serverEdges = data[1] as KartaEdge[];
                const serverStorableContextData = data[2]; // This should match StorableContext structure

                // Basic validation for StorableContext structure (can be more robust)
                if (typeof serverStorableContextData?.id !== 'string' || !Array.isArray(serverStorableContextData?.viewNodes)) {
                     console.error('[ServerAdapter] Invalid StorableContextData structure received from server:', serverStorableContextData);
                     return undefined;
                }
                
                const storableContext: StorableContext = {
                    id: serverStorableContextData.id,
                    viewNodes: serverStorableContextData.viewNodes,
                    viewportSettings: serverStorableContextData.viewportSettings,
                };

                console.log(`[ServerAdapter] Successfully fetched context bundle for path "${contextPath}"`);
                return {
                    nodes: serverNodes,
                    edges: serverEdges,
                    storableContext: storableContext,
                };
            } else {
                console.error('[ServerAdapter] Unexpected response structure from server:', data);
                return undefined;
            }
        } catch (error) {
            console.error(`[ServerAdapter] Network or parsing error fetching context bundle for path "${contextPath}":`, error);
            return undefined;
        }
    }

    // --- PersistenceService Implementation ---

    async getContext(contextPath: NodeId): Promise<StorableContext | undefined> {
        console.log(`[ServerAdapter] getContext called for path: ${contextPath}`);
        const bundle = await this.loadContextBundleByPath(contextPath);
        if (bundle) {
            console.log(`[ServerAdapter] getContext returning storableContext for path: ${contextPath}`);
            return bundle.storableContext;
        }
        console.warn(`[ServerAdapter] getContext failed to load bundle for path: ${contextPath}`);
        return undefined;
    }

    // --- Stubbed Methods ---

    async saveNode(node: DataNode): Promise<void> {
        console.warn('[ServerAdapter.saveNode] Not implemented');
        return Promise.resolve();
    }

    async getNode(nodeId: string): Promise<DataNode | undefined> {
        console.warn(`[ServerAdapter.getNode] Not implemented for ID: ${nodeId}`);
        return Promise.resolve(undefined);
    }

    async deleteNode(nodeId: string): Promise<void> {
        console.warn(`[ServerAdapter.deleteNode] Not implemented for ID: ${nodeId}`);
        return Promise.resolve();
    }

    async getNodes(): Promise<DataNode[]> {
        console.warn('[ServerAdapter.getNodes] Not implemented');
        return Promise.resolve([]);
    }

    async checkNameExists(name: string): Promise<boolean> {
        console.warn(`[ServerAdapter.checkNameExists] Not implemented for name: ${name}`);
        return Promise.resolve(false);
    }

    async getDataNodesByIds(nodeIds: NodeId[]): Promise<Map<NodeId, DataNode>> {
        console.warn(`[ServerAdapter.getDataNodesByIds] Not implemented for IDs: ${nodeIds.join(', ')}`);
        return Promise.resolve(new Map());
    }

    async getAllNodePaths(): Promise<string[]> {
        console.warn('[ServerAdapter.getAllNodePaths] Not implemented');
        return Promise.resolve([]);
    }

    async getDataNodeByPath(path: string): Promise<DataNode | undefined> {
        console.warn(`[ServerAdapter.getDataNodeByPath] Not implemented for path: ${path}`);
        return Promise.resolve(undefined);
    }

    async saveEdge(edge: KartaEdge): Promise<void> {
        console.warn('[ServerAdapter.saveEdge] Not implemented');
        return Promise.resolve();
    }

    async getEdge(edgeId: string): Promise<KartaEdge | undefined> {
        console.warn(`[ServerAdapter.getEdge] Not implemented for ID: ${edgeId}`);
        return Promise.resolve(undefined);
    }

    async getEdges(): Promise<KartaEdge[]> {
        console.warn('[ServerAdapter.getEdges] Not implemented');
        return Promise.resolve([]);
    }

    async deleteEdge(edgeId: string): Promise<void> {
        console.warn(`[ServerAdapter.deleteEdge] Not implemented for ID: ${edgeId}`);
        return Promise.resolve();
    }

    async loadEdges(): Promise<KartaEdge[]> {
        console.warn('[ServerAdapter.loadEdges] Not implemented');
        return Promise.resolve([]);
    }

    async getEdgesByNodeIds(nodeIds: NodeId[]): Promise<Map<string, KartaEdge>> {
        console.warn(`[ServerAdapter.getEdgesByNodeIds] Not implemented for IDs: ${nodeIds.join(', ')}`);
        return Promise.resolve(new Map());
    }

    async saveContext(context: Context): Promise<void> {
        console.warn(`[ServerAdapter.saveContext] Not implemented for context ID: ${context.id}`);
        return Promise.resolve();
    }

    async getAllContextIds(): Promise<NodeId[]> {
        console.warn('[ServerAdapter.getAllContextIds] Not implemented');
        return Promise.resolve([]);
    }

    async deleteContext(contextId: NodeId): Promise<void> {
        console.warn(`[ServerAdapter.deleteContext] Not implemented for ID: ${contextId}`);
        return Promise.resolve();
    }

    async getAllContextPaths(): Promise<Map<NodeId, string>> {
        console.warn('[ServerAdapter.getAllContextPaths] Not implemented');
        return Promise.resolve(new Map());
    }

    async saveAsset(assetId: string, assetData: AssetData): Promise<void> {
        console.warn(`[ServerAdapter.saveAsset] Not implemented for asset ID: ${assetId}`);
        return Promise.resolve();
    }

    async getAsset(assetId: string): Promise<AssetData | undefined> {
        console.warn(`[ServerAdapter.getAsset] Not implemented for asset ID: ${assetId}`);
        return Promise.resolve(undefined);
    }

    async deleteAsset(assetId: string): Promise<void> {
        console.warn(`[ServerAdapter.deleteAsset] Not implemented for asset ID: ${assetId}`);
        return Promise.resolve();
    }

    async getAssetObjectUrl(assetId: string): Promise<string | null> {
        console.warn(`[ServerAdapter.getAssetObjectUrl] Not implemented for asset ID: ${assetId}`);
        return Promise.resolve(null);
    }
}