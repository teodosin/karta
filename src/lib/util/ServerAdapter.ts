import type {
    DataNode,
    KartaEdge,
    NodeId,
    StorableContext,
    AssetData,
    Context, // For saveContext, though it will be stubbed
    ContextBundle,
    StorableViewNode,
    StorableViewportSettings,
} from '../types/types';
import type { PersistenceService } from './PersistenceService';

const SERVER_BASE_URL = 'http://localhost:7370'; // As determined

// Helper interfaces for expected server data structures
interface ServerAttribute {
    name: string;
    value: any; // Represents various AttrValue types (string, number, boolean, etc.)
}

interface ServerNtypeObject {
    type_path: string;
    version: string;
}

interface ServerDataNode {
    uuid: string;
    ntype: ServerNtypeObject; // Updated to reflect actual structure
    created_time: any; // Can be number or object { secs_since_epoch, nanos_since_epoch }
    modified_time: any; // Can be number or object { secs_since_epoch, nanos_since_epoch }
    path: string; // Path is guaranteed to be a string from the server
    name: string; // Will be part of attributes.name
    attributes: ServerAttribute[];
    alive: boolean; // Added to match server output
}

interface ServerKartaEdge {
    uuid: string; // Expected to be provided by the server
    source: string; // NodeId (UUID string)
    target: string; // NodeId (UUID string)
    contains: boolean;
    attributes: ServerAttribute[];
}

interface ServerViewNode {
    uuid: string; // NodeId
    relX: number; // Changed from x
    relY: number; // Changed from y
    width?: number;
    height?: number;
    relScale?: number; // Added to match server output
    rotation?: number;
    zIndex?: number;
    is_name_visible?: boolean;
    attributes: ServerAttribute[];
}

interface ServerContextSettings {
    zoom_scale: number;
    offsetX: number;
    offsetY: number;
    // any other settings from Rust's ContextSettings
}

interface ServerContext {
    focal: string; // NodeId (UUID string) of the focal node
    nodes: ServerViewNode[]; // These are the ViewNodes for the context
    settings: ServerContextSettings;
}

function transformServerAttributesToRecord(serverAttributes: ServerAttribute[]): Record<string, any> {
    const record: Record<string, any> = {};
    if (Array.isArray(serverAttributes)) {
        for (const attr of serverAttributes) {
            record[attr.name] = attr.value;
        }
    }
    return record;
}


export class ServerAdapter implements PersistenceService {
    constructor() {
        // Initialization if needed in the future
    }

    /**
     * Fetches the complete bundle of data required to render a context from the server.
     * This includes DataNodes, KartaEdges, and the StorableContext (layout).
     * @param contextPath The relative path of the context from the vault root.
     * @returns A promise that resolves with a ContextBundle, or undefined on error.
     */
    async loadContextBundle(contextPath: string): Promise<ContextBundle | undefined> {
        const encodedPath = encodeURIComponent(contextPath);
        const url = `${SERVER_BASE_URL}/ctx/${encodedPath}`;

        try {
            const response = await fetch(url);

            if (!response.ok) {
                console.error(`[ServerAdapter] Error fetching context bundle for path "${contextPath}". Status: ${response.status} ${response.statusText}`);
                const errorBody = await response.text();
                console.error(`[ServerAdapter] Error body: ${errorBody}`);
                return undefined;
            }

            const serverResponse = await response.json();

            // Expected server response: [ServerDataNode[], ServerKartaEdge[], ServerContext]
            if (Array.isArray(serverResponse) && serverResponse.length === 3) {
                const serverDataNodes = serverResponse[0] as ServerDataNode[];
                const serverKartaEdges = serverResponse[1] as ServerKartaEdge[];
                const serverContextData = serverResponse[2] as ServerContext;

                // Transform ServerDataNode[] to DataNode[]
                const clientDataNodes: DataNode[] = serverDataNodes.map(sNode => {
                    const attributes = transformServerAttributesToRecord(sNode.attributes);
                    attributes['name'] = sNode.name; // Ensure name is part of attributes
                    
                    // Handle potential object structure for time fields
                    const createdTime = typeof sNode.created_time === 'number'
                                        ? sNode.created_time
                                        : (sNode.created_time?.secs_since_epoch ?? 0);
                    const modifiedTime = typeof sNode.modified_time === 'number'
                                         ? sNode.modified_time
                                         : (sNode.modified_time?.secs_since_epoch ?? 0);

                    return {
                        id: sNode.uuid,
                        ntype: sNode.ntype.type_path, // Extract type_path string
                        createdAt: createdTime * 1000, // Convert seconds to milliseconds
                        modifiedAt: modifiedTime * 1000, // Convert seconds to milliseconds
                        path: sNode.path, // Path is guaranteed to be a string
                        attributes: attributes,
                        alive: sNode.alive, // Map the top-level alive field
                    };
                });

                // Transform ServerKartaEdge[] to KartaEdge[]
                const clientKartaEdges: KartaEdge[] = serverKartaEdges.map(sEdge => {
                    const attributes = transformServerAttributesToRecord(sEdge.attributes);
                    attributes['contains'] = sEdge.contains; // Ensure contains is part of attributes
                    return {
                        id: sEdge.uuid, // Directly use the UUID from server
                        source: sEdge.source,
                        target: sEdge.target,
                        attributes: attributes,
                    };
                });

                // Transform ServerContext to StorableContext
                const clientViewNodes: [NodeId, StorableViewNode][] = serverContextData.nodes.map(sViewNode => {
                    const attributes = transformServerAttributesToRecord(sViewNode.attributes);
                    if (sViewNode.is_name_visible !== undefined) {
                        attributes['isNameVisible'] = sViewNode.is_name_visible;
                    }
                    // Provide default values for potentially undefined properties from server
                    // These defaults should align with how StorableViewNode is defined or typically initialized.
                    // For now, using common defaults. These might need adjustment based on actual StorableViewNode expectations.
                    const defaultWidth = 100; // Example default
                    const defaultHeight = 100; // Example default

                    const storableViewNode: StorableViewNode = {
                        id: sViewNode.uuid,
                        relX: sViewNode.relX, // Use sViewNode.relX
                        relY: sViewNode.relY, // Use sViewNode.relY
                        width: sViewNode.width ?? defaultWidth,
                        height: sViewNode.height ?? defaultHeight,
                        rotation: sViewNode.rotation ?? 0,
                        relScale: sViewNode.relScale ?? 1, // Use sViewNode.relScale, default to 1 if not present
                        attributes: attributes,
                    };
                    return [sViewNode.uuid, storableViewNode];
                });

                const clientViewportSettings: StorableViewportSettings = {
                    scale: (serverContextData.settings?.zoom_scale ?? 1.0) > 0.001 ? (serverContextData.settings?.zoom_scale ?? 1.0) : 1.0,
                    relPosX: Number(serverContextData.settings?.offsetX) || 0,
                    relPosY: Number(serverContextData.settings?.offsetY) || 0,
                };

                const clientStorableContext: StorableContext = {
                    id: serverContextData.focal, // focal node ID
                    viewNodes: clientViewNodes,
                    viewportSettings: clientViewportSettings,
                };

                const dataNodeDebugPrint: string[] = clientDataNodes.map((node) => {
                    return node.path
                })
                console.log(dataNodeDebugPrint);
                
                return {
                    nodes: clientDataNodes,
                    edges: clientKartaEdges,
                    storableContext: clientStorableContext,
                };
            } else {
                console.error('[ServerAdapter] Unexpected response structure from server:', serverResponse);
                return undefined;
            }
        } catch (error) {
            console.error(`[ServerAdapter] Network or parsing error fetching context bundle for path "${contextPath}":`, error);
            return undefined;
        }
    }

    // --- PersistenceService Implementation ---

    async getContext(contextPath: NodeId): Promise<StorableContext | undefined> {
        // contextPath is used as the identifier for loadContextBundle
        const bundle = await this.loadContextBundle(contextPath);
        if (bundle) {
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