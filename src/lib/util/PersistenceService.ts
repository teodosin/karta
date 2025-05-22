import type { AssetData, Context, DataNode, KartaEdge, NodeId, StorableContext } from "$lib/types/types";

export interface PersistenceService {
    // Node methods
    saveNode(node: DataNode): Promise<void>;
    getNode(nodeId: string): Promise<DataNode | undefined>; // Use DataNode
    deleteNode(nodeId: string): Promise<void>;
    getNodes(): Promise<DataNode[]>;
    checkNameExists(name: string): Promise<boolean>; // Re-add checkNameExists signature
    getDataNodesByIds(nodeIds: NodeId[]): Promise<Map<NodeId, DataNode>>; // Added this method
    getAllNodePaths(): Promise<string[]>; // Added this method
    getDataNodeByPath(path: string): Promise<DataNode | undefined>; // Added this method

    // Edge methods
    saveEdge(edge: KartaEdge): Promise<void>;
    getEdge(edgeId: string): Promise<KartaEdge | undefined>;
    getEdges(): Promise<KartaEdge[]>;
    deleteEdge(edgeId: string): Promise<void>;
    loadEdges(): Promise<KartaEdge[]>; // Added this method
    getEdgesByNodeIds(nodeIds: NodeId[]): Promise<Map<string, KartaEdge>>; // Added this method

    // Context methods (no longer pass focalAbsTransform to saveContext)
    // saveContext still takes the in-memory Context (with Tweens)
    saveContext(context: Context): Promise<void>;
    // getContext returns the StorableContext read from DB
    getContext(contextId: NodeId): Promise<StorableContext | undefined>;
    getAllContextIds(): Promise<NodeId[]>;
    deleteContext(contextId: NodeId): Promise<void>; // Added this method
    getAllContextPaths(): Promise<Map<NodeId, string>>; // Added this method

    // Asset methods
    saveAsset(assetId: string, assetData: AssetData): Promise<void>; // Added this method
    getAsset(assetId: string): Promise<AssetData | undefined>; // Added this method
    deleteAsset(assetId: string): Promise<void>; // Added this method
    getAssetObjectUrl(assetId: string): Promise<string | null>; // Added this method
}