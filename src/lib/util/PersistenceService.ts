import type { AssetData, Context, ContextBundle, DataNode, EdgeDeletionPayload, KartaEdge, KartaEdgeCreationPayload, NodeId, StorableContext } from "$lib/types/types";

export interface PersistenceService {
    // Node methods
    saveNode(node: DataNode): Promise<void>;
    getNode(nodeId: string): Promise<DataNode | undefined>;
    deleteNode(nodeId: string): Promise<void>;
    getNodes(): Promise<DataNode[]>;
    checkNameExists(name: string): Promise<boolean>;
    getDataNodesByIds(nodeIds: NodeId[]): Promise<Map<NodeId, DataNode>>;
    getAllNodePaths(): Promise<string[]>;
    getDataNodeByPath(path: string): Promise<DataNode | undefined>;

    // Edge methods
    createEdges(edges: KartaEdgeCreationPayload[]): Promise<KartaEdge[] | undefined>;
    getEdge(edgeId: string): Promise<KartaEdge | undefined>;
    getEdges(): Promise<KartaEdge[]>;
    deleteEdges(payload: EdgeDeletionPayload[]): Promise<void>;
    reconnectEdge(old_from: NodeId, old_to: NodeId, new_from: NodeId, new_to: NodeId, new_from_path: string, new_to_path: string): Promise<KartaEdge | undefined>;
    loadEdges(): Promise<KartaEdge[]>;
    getEdgesByNodeIds(nodeIds: NodeId[]): Promise<Map<string, KartaEdge>>;

    // Context methods
    // saveContext still takes the in-memory Context (with Tweens)
    saveContext(context: Context): Promise<void>;
    // getContext returns the StorableContext read from DB
    getContext(contextId: NodeId): Promise<StorableContext | undefined>;
    getAllContextIds(): Promise<NodeId[]>;
    deleteContext(contextId: NodeId): Promise<void>;
    getAllContextPaths(): Promise<Map<NodeId, string>>;
    loadContextBundle(identifier: string): Promise<ContextBundle | undefined>;

    // Asset methods
    saveAsset(assetId: string, assetData: AssetData): Promise<void>;
    getAsset(assetId: string): Promise<AssetData | undefined>;
    deleteAsset(assetId: string): Promise<void>;
    getAssetObjectUrl(assetId: string): Promise<string | null>;
}