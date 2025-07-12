import type {
    DataNode,
    KartaEdge,
    NodeId,
    StorableContext,
    AssetData,
    Context,
    ContextBundle,
    StorableViewNode,
    StorableViewportSettings,
    KartaSettings,
} from '../types/types';
import type { PersistenceService } from './PersistenceService';

const SERVER_BASE_URL = 'http://localhost:7370';

// Helper interfaces for expected server data structures
interface ServerAttribute {
    name: string;
    value: any;
}

interface ServerNtypeObject {
    type_path: string;
    version: string;
}

interface ServerDataNode {
    uuid: string;
    ntype: ServerNtypeObject;
    created_time: any;
    modified_time: any;
    path: string;
    name: string;
    attributes: ServerAttribute[];
    alive: boolean;
}

interface ServerKartaEdge {
    uuid: string;
    source: string;
    target: string;
    contains: boolean;
    attributes: ServerAttribute[];
}

interface ServerViewNode {
    uuid: string;
    relX: number;
    relY: number;
    width?: number;
    height?: number;
    relScale?: number;
    rotation?: number;
    zIndex?: number;
    is_name_visible?: boolean;
    status?: 'generated' | 'modified';
    attributes: ServerAttribute[];
}

interface ServerContextSettings {
    zoom_scale: number;
    view_rel_pos_x: number;
    view_rel_pos_y: number;
}

interface ServerContext {
    focal: string;
    nodes: ServerViewNode[];
    settings: ServerContextSettings;
}

function transformServerAttributesToRecord(serverAttributes: ServerAttribute[]): Record<string, any> {
    const record: Record<string, any> = {};
    if (Array.isArray(serverAttributes)) {
        for (const attr of serverAttributes) {
            if (typeof attr.value === 'object' && attr.value !== null) {
                 const keys = Object.keys(attr.value);
                 if (keys.length === 1) {
                     record[attr.name] = attr.value[keys[0]];
                 } else {
                    record[attr.name] = attr.value;
                 }
            } else {
                record[attr.name] = attr.value;
            }
        }
    }
    return record;
}

/**
 * Transforms a client-side DataNode's attributes into the format expected by the server.
 * @param attributes The client-side attributes record.
 * @returns An array of ServerAttribute objects.
 */
function transformAttributesToServerFormat(attributes: Record<string, any>): ServerAttribute[] {
    return Object.entries(attributes)
        .filter(([key, value]) => value !== undefined && value !== null)
        .map(([name, value]) => {
            // The backend expects a tagged union format.
            let taggedValue: any;
            switch (typeof value) {
                case 'string':
                    taggedValue = { String: value };
                    break;
                case 'number':
                    // Using Float as a general case for numbers.
                    taggedValue = { Float: value };
                    break;
                case 'boolean':
                    taggedValue = { UInt: value ? 1 : 0 };
                    break;
                default:
                    // For complex objects, serialize them as a JSON string.
                    taggedValue = { String: JSON.stringify(value) };
                    break;
            }
            return { name, value: taggedValue };
        });
}


export class ServerAdapter implements PersistenceService {
    constructor() {}

    async createNode(node: DataNode, parentPath: string): Promise<DataNode | undefined> {
        const url = `${SERVER_BASE_URL}/api/nodes`;
        const payload = {
            name: node.attributes['name'] || 'Unnamed Node',
            ntype: { type_path: node.ntype, version: "0.1.0" },
            parent_path: parentPath,
            attributes: transformAttributesToServerFormat(node.attributes),
        };

        try {
            const response = await fetch(url, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload),
            });

            if (!response.ok) {
                const errorBody = await response.text();
                console.error(`[ServerAdapter.createNode] Error creating node. Status: ${response.status}`, errorBody);
                throw new Error(`Server responded with status ${response.status}`);
            }

            const serverNode: ServerDataNode = await response.json();
            const attributes = transformServerAttributesToRecord(serverNode.attributes);
            attributes['name'] = serverNode.name;

            return {
                id: serverNode.uuid,
                ntype: serverNode.ntype.type_path,
                createdAt: (serverNode.created_time?.secs_since_epoch ?? 0) * 1000,
                modifiedAt: (serverNode.modified_time?.secs_since_epoch ?? 0) * 1000,
                path: serverNode.path,
                attributes: attributes,
                isSearchable: attributes['isSearchable'] ?? true,
            };

        } catch (error) {
            console.error(`[ServerAdapter.createNode] Network error creating node:`, error);
            throw error;
        }
    }

    async updateNode(node: DataNode): Promise<DataNode | undefined> {
        const url = `${SERVER_BASE_URL}/api/nodes/${node.id}`;
        const payload = {
             attributes: transformAttributesToServerFormat(node.attributes),
        };

        try {
            const response = await fetch(url, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload),
            });

            if (!response.ok) {
                const errorBody = await response.text();
                console.error(`[ServerAdapter.updateNode] Error updating node ${node.id}. Status: ${response.status}`, errorBody);
                throw new Error(`Server responded with status ${response.status}`);
            }
            
            const serverNode: ServerDataNode = await response.json();
            const attributes = transformServerAttributesToRecord(serverNode.attributes);
            attributes['name'] = serverNode.name;

            return {
                id: serverNode.uuid,
                ntype: serverNode.ntype.type_path,
                createdAt: (serverNode.created_time?.secs_since_epoch ?? 0) * 1000,
                modifiedAt: (serverNode.modified_time?.secs_since_epoch ?? 0) * 1000,
                path: serverNode.path,
                attributes: attributes,
                isSearchable: attributes['isSearchable'] ?? true,
            };

        } catch (error) {
            console.error(`[ServerAdapter.updateNode] Network error updating node ${node.id}:`, error);
            throw error;
        }
    }

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

            if (Array.isArray(serverResponse) && serverResponse.length === 3) {
                const serverDataNodes = serverResponse[0] as ServerDataNode[];
                const serverKartaEdges = serverResponse[1] as ServerKartaEdge[];
                const serverContextData = serverResponse[2] as ServerContext;

                const clientDataNodes: DataNode[] = serverDataNodes.map(sNode => {
                    const attributes = transformServerAttributesToRecord(sNode.attributes);
                    attributes['name'] = sNode.name;
                    
                    const createdTime = typeof sNode.created_time === 'number'
                                        ? sNode.created_time
                                        : (sNode.created_time?.secs_since_epoch ?? 0);
                    const modifiedTime = typeof sNode.modified_time === 'number'
                                         ? sNode.modified_time
                                         : (sNode.modified_time?.secs_since_epoch ?? 0);

                    return {
                        id: sNode.uuid,
                        ntype: sNode.ntype.type_path,
                        createdAt: createdTime * 1000,
                        modifiedAt: modifiedTime * 1000,
                        path: sNode.path,
                        attributes: attributes,
                        isSearchable: attributes['isSearchable'] ?? true,
                    };
                });

                const clientKartaEdges: KartaEdge[] = serverKartaEdges.map(sEdge => ({
                    id: sEdge.uuid,
                    source: sEdge.source,
                    target: sEdge.target,
                    attributes: transformServerAttributesToRecord(sEdge.attributes),
                }));

                const clientViewNodes: [NodeId, StorableViewNode][] = serverContextData.nodes.map(sViewNode => {
                    const attributes = transformServerAttributesToRecord(sViewNode.attributes);
                    if (sViewNode.is_name_visible !== undefined) {
                        attributes['isNameVisible'] = sViewNode.is_name_visible;
                    }
                    
                    const storableViewNode: StorableViewNode = {
                        id: sViewNode.uuid,
                        relX: sViewNode.relX,
                        relY: sViewNode.relY,
                        width: sViewNode.width ?? 100,
                        height: sViewNode.height ?? 100,
                        rotation: sViewNode.rotation ?? 0,
                        relScale: sViewNode.relScale ?? 1,
                        status: sViewNode.status?.toLowerCase() as 'generated' | 'modified' ?? 'modified',
                        attributes: attributes,
                    };
                    return [sViewNode.uuid, storableViewNode];
                });

                const clientViewportSettings: StorableViewportSettings = {
                    scale: (serverContextData.settings?.zoom_scale ?? 1.0) > 0.001 ? (serverContextData.settings?.zoom_scale ?? 1.0) : 1.0,
                    relPosX: Number(serverContextData.settings?.view_rel_pos_x) || 0,
                    relPosY: Number(serverContextData.settings?.view_rel_pos_y) || 0,
                };

                const clientStorableContext: StorableContext = {
                    id: serverContextData.focal,
                    viewNodes: clientViewNodes,
                    viewportSettings: clientViewportSettings,
                };
                
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

    async getContext(contextPath: NodeId): Promise<StorableContext | undefined> {
        const bundle = await this.loadContextBundle(contextPath);
        return bundle?.storableContext;
    }

    async saveContext(context: Context): Promise<void> {
        const modifiedViewNodes = Array.from(context.viewNodes.values()).filter(vn => vn.status === 'modified');

        if (modifiedViewNodes.length === 0) {
            return Promise.resolve();
        }

        const focalNode = context.viewNodes.get(context.id);
        if (!focalNode) {
            console.error(`[ServerAdapter.saveContext] Focal node ${context.id} not found in context.`);
            return Promise.reject(new Error('Focal node not found'));
        }
        const focalState = focalNode.state.current;

        const serverViewNodes = modifiedViewNodes.map(viewNode => {
            const { id, state, attributes, status } = viewNode;
            const { x, y, scale, rotation, width, height } = state.current;

            const relX = id === context.id ? 0 : (x - focalState.x) / focalState.scale;
            const relY = id === context.id ? 0 : (y - focalState.y) / focalState.scale;
            const relScale = id === context.id ? 1 : scale / focalState.scale;

            const isNameVisible = attributes?.['isNameVisible'] ?? true;

            const serverAttributes: ServerAttribute[] = attributes
                ? transformAttributesToServerFormat(attributes)
                : [];

            return {
                uuid: id,
                status: status.charAt(0).toUpperCase() + status.slice(1),
                is_name_visible: isNameVisible,
                relX,
                relY,
                width,
                height,
                relScale,
                rotation,
                attributes: serverAttributes,
            };
        });

        const { scale: currentScale, posX: absPosX, posY: absPosY } = context.viewportSettings ?? { scale: 1.0, posX: 0, posY: 0 };
        const relPosX = absPosX + (focalState.x * currentScale);
        const relPosY = absPosY + (focalState.y * currentScale);

        const payload = {
            karta_version: "0.1.0",
            focal: context.id,
            nodes: serverViewNodes,
            settings: {
                zoom_scale: currentScale,
                view_rel_pos_x: relPosX,
                view_rel_pos_y: relPosY,
            }
        };

        const url = `${SERVER_BASE_URL}/api/ctx/${context.id}`;
        try {
            console.log(`[ServerAdapter.saveContext] Saving context ${context.id}. Payload:`, JSON.stringify(payload, null, 2));
            const response = await fetch(url, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(payload),
            });

            if (!response.ok) {
                const errorBody = await response.text();
                console.error(`[ServerAdapter.saveContext] Error saving context ${context.id}. Status: ${response.status}`, errorBody);
                throw new Error(`Server responded with status ${response.status}`);
            }

            console.log(`[ServerAdapter.saveContext] Successfully saved layout for ${modifiedViewNodes.length} nodes in context ${context.id}`);
        } catch (error) {
            console.error(`[ServerAdapter.saveContext] Network error saving context ${context.id}:`, error);
            throw error;
        }
    }

    // --- Stubbed Methods ---
    async saveNode(node: DataNode): Promise<void> {
        // This can be a wrapper, but for now we expect the caller to use create/update directly.
        console.warn('[ServerAdapter.saveNode] Deprecated. Use createNode or updateNode directly.');
    }
    async getNode(nodeId: string): Promise<DataNode | undefined> {
        const url = `${SERVER_BASE_URL}/api/nodes/${nodeId}`;
        try {
            const response = await fetch(url);
            if (!response.ok) {
                if (response.status !== 404) {
                    console.error(`[ServerAdapter.getNode] Error fetching node ${nodeId}. Status: ${response.status}`);
                }
                return undefined;
            }
            const serverNode: ServerDataNode = await response.json();
            const attributes = transformServerAttributesToRecord(serverNode.attributes);
            attributes['name'] = serverNode.name;

            return {
                id: serverNode.uuid,
                ntype: serverNode.ntype.type_path,
                createdAt: (serverNode.created_time?.secs_since_epoch ?? 0) * 1000,
                modifiedAt: (serverNode.modified_time?.secs_since_epoch ?? 0) * 1000,
                path: serverNode.path,
                attributes: attributes,
                isSearchable: attributes['isSearchable'] ?? true,
            };
        } catch (error) {
            console.error(`[ServerAdapter.getNode] Network error fetching node ${nodeId}:`, error);
            return undefined;
        }
    }
    async deleteNode(nodeId: string): Promise<void> { console.warn(`[ServerAdapter.deleteNode] Not implemented for ID: ${nodeId}`); }
    async getNodes(): Promise<DataNode[]> { console.warn('[ServerAdapter.getNodes] Not implemented'); return []; }
    async checkNameExists(name: string): Promise<boolean> { console.warn(`[ServerAdapter.checkNameExists] Not implemented for name: ${name}`); return false; }
    async getDataNodesByIds(nodeIds: NodeId[]): Promise<Map<NodeId, DataNode>> { console.warn(`[ServerAdapter.getDataNodesByIds] Not implemented`); return new Map(); }
    async getAllNodePaths(): Promise<string[]> { console.warn('[ServerAdapter.getAllNodePaths] Not implemented'); return []; }
    async getDataNodeByPath(path: string): Promise<DataNode | undefined> { console.warn(`[ServerAdapter.getDataNodeByPath] Not implemented for path: ${path}`); return undefined; }
    async saveEdge(edge: KartaEdge): Promise<void> { console.warn('[ServerAdapter.saveEdge] Not implemented'); }
    async getEdge(edgeId: string): Promise<KartaEdge | undefined> { console.warn(`[ServerAdapter.getEdge] Not implemented for ID: ${edgeId}`); return undefined; }
    async getEdges(): Promise<KartaEdge[]> { console.warn('[ServerAdapter.getEdges] Not implemented'); return []; }
    async deleteEdge(edgeId: string): Promise<void> { console.warn(`[ServerAdapter.deleteEdge] Not implemented for ID: ${edgeId}`); }
    async loadEdges(): Promise<KartaEdge[]> { console.warn('[ServerAdapter.loadEdges] Not implemented'); return []; }
    async getEdgesByNodeIds(nodeIds: NodeId[]): Promise<Map<string, KartaEdge>> { console.warn(`[ServerAdapter.getEdgesByNodeIds] Not implemented`); return new Map(); }
    async getAllContextIds(): Promise<NodeId[]> { console.warn('[ServerAdapter.getAllContextIds] Not implemented'); return []; }
    async deleteContext(contextId: NodeId): Promise<void> { console.warn(`[ServerAdapter.deleteContext] Not implemented for ID: ${contextId}`); }
    async getAllContextPaths(): Promise<Map<NodeId, string>> { console.warn('[ServerAdapter.getAllContextPaths] Not implemented'); return new Map(); }
    async saveAsset(assetId: string, assetData: AssetData): Promise<void> { console.warn(`[ServerAdapter.saveAsset] Not implemented for asset ID: ${assetId}`); }
    async getAsset(assetId: string): Promise<AssetData | undefined> { console.warn(`[ServerAdapter.getAsset] Not implemented for asset ID: ${assetId}`); return undefined; }
    async deleteAsset(assetId: string): Promise<void> { console.warn(`[ServerAdapter.deleteAsset] Not implemented for asset ID: ${assetId}`); }
    async getAssetObjectUrl(assetId: string): Promise<string | null> { console.warn(`[ServerAdapter.getAssetObjectUrl] Not implemented for asset ID: ${assetId}`); return null; }

    async getSettings(): Promise<KartaSettings | undefined> {
        const url = `${SERVER_BASE_URL}/api/settings`;
        try {
            const response = await fetch(url);
            if (!response.ok) {
                if (response.status !== 404) {
                    console.error(`[ServerAdapter.getSettings] Error fetching settings. Status: ${response.status}`);
                }
                return undefined;
            }
            return await response.json();
        } catch (error) {
            console.error(`[ServerAdapter.getSettings] Network error fetching settings:`, error);
            return undefined;
        }
    }

    async saveSettings(settings: KartaSettings): Promise<void> {
        const url = `${SERVER_BASE_URL}/api/settings`;
        try {
            const response = await fetch(url, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(settings),
            });

            if (!response.ok) {
                const errorBody = await response.text();
                console.error(`[ServerAdapter.saveSettings] Error saving settings. Status: ${response.status}`, errorBody);
                throw new Error(`Server responded with status ${response.status}`);
            }
        } catch (error) {
            console.error(`[ServerAdapter.saveSettings] Network error saving settings:`, error);
            throw error;
        }
    }
}