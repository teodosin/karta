import { KARTA_VERSION } from '$lib/constants';
import { apiLogger } from '$lib/debug/loggers';
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
    EdgeId,
    EdgeDeletionPayload,
    MoveOperation,
    MoveNodesResponse,
    DeleteNodesResponse,
    SearchQuery,
    SearchResponse,
    BundleTreeResponse,
    ExportBundleRequest,
    ExportBundleResponse,
} from '../types/types';
import type { KartaEdgeCreationPayload } from '$lib/types/types';
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
            ntype: { type_path: node.ntype, version: KARTA_VERSION },
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
            // Extract name from path since backend no longer has name field
            const pathSegments = serverNode.path.split('/');
            const name = pathSegments[pathSegments.length - 1] || 'root';
            attributes['name'] = name;

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
            
            const updateResponse = await response.json();
            
            // Handle both old and new response formats for compatibility
            let serverNode: ServerDataNode;
            let affectedNodes: ServerDataNode[] = [];
            
            if (updateResponse.updated_node) {
                // New format with UpdateNodeResponse
                serverNode = updateResponse.updated_node;
                affectedNodes = updateResponse.affected_nodes || [];
            } else {
                // Old format with direct DataNode
                serverNode = updateResponse;
            }
            
            const attributes = transformServerAttributesToRecord(serverNode.attributes);
            // Extract name from path since backend no longer has name field
            const pathSegments = serverNode.path.split('/');
            const name = pathSegments[pathSegments.length - 1] || 'root';
            attributes['name'] = name;

            const updatedNode: DataNode = {
                id: serverNode.uuid,
                ntype: serverNode.ntype.type_path,
                createdAt: (serverNode.created_time?.secs_since_epoch ?? 0) * 1000,
                modifiedAt: (serverNode.modified_time?.secs_since_epoch ?? 0) * 1000,
                path: serverNode.path,
                attributes: attributes,
                isSearchable: attributes['isSearchable'] ?? true,
            };

            // If there are affected nodes (descendants), update them in the store too
            if (affectedNodes.length > 0) {
                const { nodes } = await import('$lib/karta/NodeStore');
                const affectedDataNodes: DataNode[] = affectedNodes.map(sNode => {
                    const attrs = transformServerAttributesToRecord(sNode.attributes);
                    // Extract name from the path
                    const nameFromPath = sNode.path.split('/').pop() || (sNode.path === '' ? 'root' : '');
                    attrs['name'] = nameFromPath;
                    return {
                        id: sNode.uuid,
                        ntype: sNode.ntype.type_path,
                        createdAt: (sNode.created_time?.secs_since_epoch ?? 0) * 1000,
                        modifiedAt: (sNode.modified_time?.secs_since_epoch ?? 0) * 1000,
                        path: sNode.path,
                        attributes: attrs,
                        isSearchable: attrs['isSearchable'] ?? true,
                    };
                });
                
                // Update all affected nodes in the store
                nodes.update(nodeMap => {
                    affectedDataNodes.forEach(affectedNode => {
                        nodeMap.set(affectedNode.id, affectedNode);
                    });
                    return nodeMap;
                });
                
                console.log(`[ServerAdapter.updateNode] Updated ${affectedNodes.length} affected descendant nodes in store`);
            }

            return updatedNode;

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

                // Log all DataNode paths in the incoming context bundle
                console.log(`[ServerAdapter.loadContextBundle] Context "${contextPath}" contains ${serverDataNodes.length} DataNodes:`);
                serverDataNodes.forEach(sNode => {
                    console.log(`  - ID: ${sNode.uuid}, Path: "${sNode.path}"`);
                });

                const clientDataNodes: DataNode[] = serverDataNodes.map(sNode => {
                    const attributes = transformServerAttributesToRecord(sNode.attributes);
                    // Extract name from the path
                    const nameFromPath = sNode.path.split('/').pop() || (sNode.path === '' ? 'root' : '');
                    attributes['name'] = nameFromPath;
                    
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

                const clientKartaEdges: KartaEdge[] = serverKartaEdges.map(sEdge => {
                    const attributes = transformServerAttributesToRecord(sEdge.attributes);
                    const clientEdge = {
                        id: sEdge.uuid,
                        source: sEdge.source,
                        target: sEdge.target,
                        attributes: attributes,
                        contains: sEdge.contains,
                    };
                    return clientEdge;
                });

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

        console.log(`[ServerAdapter.saveContext] Context ${context.id} has ${context.viewNodes.size} total ViewNodes, ${modifiedViewNodes.length} modified:`);
        modifiedViewNodes.forEach(vn => {
            const dataNode = context.viewNodes.get(vn.id);
            console.log(`  - Modified ViewNode ID: ${vn.id} (status: ${vn.status})`);
        });

        if (modifiedViewNodes.length === 0) {
            console.log(`[ServerAdapter.saveContext] No modified ViewNodes to save for context ${context.id}`);
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
            // Extract name from the path
            const nameFromPath = serverNode.path.split('/').pop() || (serverNode.path === '' ? 'root' : '');
            attributes['name'] = nameFromPath;

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

    /**
     * Rename a node by its path using the dedicated rename endpoint.
     * This is useful for unindexed files that don't have UUIDs yet.
     */
    async renameNode(path: string, newName: string): Promise<DataNode | undefined> {
        const url = `${SERVER_BASE_URL}/api/nodes/rename`;
        console.log(`[ServerAdapter.renameNode] Renaming node at path "${path}" to "${newName}"`);
        
        const payload = {
            path: path,
            new_name: newName
        };

        try {
            const response = await fetch(url, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload),
            });

            if (!response.ok) {
                const errorBody = await response.text();
                console.error(`[ServerAdapter.renameNode] Error renaming node at path "${path}". Status: ${response.status}`, errorBody);
                throw new Error(`Server responded with status ${response.status}`);
            }
            
            const renameResponse = await response.json();
            const renamedNodes = renameResponse.renamed_nodes;
            
            if (!renamedNodes || renamedNodes.length === 0) {
                console.error(`[ServerAdapter.renameNode] No renamed nodes returned from server`);
                return undefined;
            }
            
            // Find the main renamed node (the one that was originally requested)
            const mainRenamedNode = renamedNodes.find((node: any) => 
                // The main node should be the one whose old path matches our request
                // Since we don't have the old path in the response, we'll take the first one
                // that has the new name we requested
                node.path.endsWith(`/${newName}`) || node.path === newName
            ) || renamedNodes[0];
            
            console.log(`[ServerAdapter.renameNode] Found main renamed node: ${mainRenamedNode.path}`);
            
            // Fetch the updated node data from the server
            const updatedNode = await this.getDataNodeByPath(mainRenamedNode.path);
            if (!updatedNode) {
                console.error(`[ServerAdapter.renameNode] Failed to fetch updated node data for path "${mainRenamedNode.path}"`);
                return undefined;
            }

            // Update all affected nodes in the store
            const { nodes } = await import('$lib/karta/NodeStore');
            nodes.update(nodeMap => {
                renamedNodes.forEach((renamedNodeInfo: any) => {
                    const nodeId = renamedNodeInfo.uuid;
                    const newPath = renamedNodeInfo.path;
                    const existingNode = nodeMap.get(nodeId);
                    
                    if (existingNode) {
                        const updatedNode = {
                            ...existingNode,
                            path: newPath,
                            attributes: {
                                ...existingNode.attributes,
                                name: newPath.split('/').pop() || 'root'
                            }
                        };
                        nodeMap.set(nodeId, updatedNode);
                    }
                });
                return nodeMap;
            });
            
            console.log(`[ServerAdapter.renameNode] Updated ${renamedNodes.length} affected nodes in store`);

            console.log(`[ServerAdapter.renameNode] Successfully renamed node to path "${updatedNode.path}"`);
            
            // Check if the renamed node is the current context's focal node
            // If so, update the lastViewedContextPath setting
            const { currentContextId } = await import('$lib/karta/ContextStore');
            const { settings, updateSettings } = await import('$lib/karta/SettingsStore');
            const { get } = await import('svelte/store');
            
            const contextId = get(currentContextId);
            if (updatedNode.id === contextId) {
                console.log(`[ServerAdapter.renameNode] Renamed node is the current context focal node, updating lastViewedContextPath`);
                try {
                    const currentSettings = get(settings);
                    if (currentSettings.savelastViewedContextPath) {
                        await updateSettings({ lastViewedContextPath: updatedNode.path });
                        console.log(`[ServerAdapter.renameNode] Updated lastViewedContextPath to: ${updatedNode.path}`);
                    }
                } catch (error) {
                    console.error('[ServerAdapter.renameNode] Error updating lastViewedContextPath:', error);
                }
            }
            
            return updatedNode;
        } catch (error) {
            console.error(`[ServerAdapter.renameNode] Network error renaming node at path "${path}":`, error);
            throw error;
        }
    }

    async deleteNodes(nodeHandles: string[]): Promise<DeleteNodesResponse> {
        console.log(`[ServerAdapter.deleteNodes] ===== STARTING SERVER DELETE REQUEST =====`);
        console.log(`[ServerAdapter.deleteNodes] Node handles to delete:`, nodeHandles);
        
        const url = `${SERVER_BASE_URL}/api/nodes`;
        const payload = {
            node_handles: nodeHandles, // Can be paths or UUIDs
            context_id: null
        };
        
        console.log(`[ServerAdapter.deleteNodes] Request URL: ${url}`);
        console.log(`[ServerAdapter.deleteNodes] Request payload:`, JSON.stringify(payload, null, 2));

        try {
            console.log(`[ServerAdapter.deleteNodes] Sending DELETE request to server...`);
            const response = await fetch(url, {
                method: 'DELETE',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload),
            });

            console.log(`[ServerAdapter.deleteNodes] Server response status: ${response.status}`);
            console.log(`[ServerAdapter.deleteNodes] Server response headers:`, Object.fromEntries(response.headers.entries()));

            if (!response.ok) {
                const errorBody = await response.text();
                console.error(`[ServerAdapter.deleteNodes] Error deleting nodes. Status: ${response.status}`, errorBody);
                throw new Error(`Server responded with status ${response.status}`);
            }

            const deleteResponse = await response.json();
            console.log(`[ServerAdapter.deleteNodes] Server delete response:`, JSON.stringify(deleteResponse, null, 2));
            console.log(`[ServerAdapter.deleteNodes] ===== SERVER DELETE REQUEST COMPLETED =====`);
            return deleteResponse;
            
        } catch (error) {
            console.error(`[ServerAdapter.deleteNodes] Network error deleting nodes:`, error);
            throw error;
        }
    }

    async getNodes(): Promise<DataNode[]> { console.warn('[ServerAdapter.getNodes] Not implemented'); return []; }
    async checkNameExists(name: string): Promise<boolean> { console.warn(`[ServerAdapter.checkNameExists] Not implemented for name: ${name}`); return false; }
    async getDataNodesByIds(nodeIds: NodeId[]): Promise<Map<NodeId, DataNode>> { console.warn(`[ServerAdapter.getDataNodesByIds] Not implemented`); return new Map(); }
    async getAllNodePaths(): Promise<string[]> { console.warn('[ServerAdapter.getAllNodePaths] Not implemented'); return []; }
    
    async getDataNodeByPath(path: string): Promise<DataNode | undefined> {
        const encodedPath = encodeURIComponent(path);
        const url = `${SERVER_BASE_URL}/api/nodes/by-path/${encodedPath}`;
        try {
            const response = await fetch(url);
            if (!response.ok) {
                if (response.status !== 404) {
                    apiLogger.error(`Error fetching node with path "${path}". Status: ${response.status}`);
                }
                return undefined;
            }
            const serverNode: ServerDataNode = await response.json();
            const attributes = transformServerAttributesToRecord(serverNode.attributes);
            // Extract name from the path
            const nameFromPath = serverNode.path.split('/').pop() || (serverNode.path === '' ? 'root' : '');
            attributes['name'] = nameFromPath;

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
            apiLogger.error(` Network error:`, error);
            return undefined;
        }
    }

    /**
     * Gets a DataNode by path and ensures it's indexed in the database.
     * This is useful when adding nodes from search results to ensure they have UUIDs.
     */
    async getAndIndexDataNodeByPath(path: string): Promise<DataNode | undefined> {
        const encodedPath = encodeURIComponent(path);
        const url = `${SERVER_BASE_URL}/api/nodes/by-path-and-index/${encodedPath}`;
        try {
            const response = await fetch(url, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
            });
            if (!response.ok) {
                if (response.status !== 404) {
                    apiLogger.error(`Error fetching and indexing node with path "${path}". Status: ${response.status}`);
                }
                return undefined;
            }
            const serverNode: ServerDataNode = await response.json();
            const attributes = transformServerAttributesToRecord(serverNode.attributes);
            // Extract name from the path
            const nameFromPath = serverNode.path.split('/').pop() || (serverNode.path === '' ? 'root' : '');
            attributes['name'] = nameFromPath;

            console.log(`[ServerAdapter.getAndIndexDataNodeByPath] Retrieved and indexed node: ${path} (UUID: ${serverNode.uuid})`);

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
            apiLogger.error(`Network error fetching and indexing node with path "${path}":`, error);
            return undefined;
        }
    }

    async createEdges(edges: KartaEdgeCreationPayload[]): Promise<KartaEdge[] | undefined> {
        const url = `${SERVER_BASE_URL}/api/edges`;
        console.log('[ServerAdapter.createEdges] Sending payload to server:', JSON.stringify(edges, null, 2));
        try {
            const response = await fetch(url, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(edges),
            });

            if (!response.ok) {
                const errorBody = await response.text();
                console.error(`[ServerAdapter.createEdges] Error creating edges. Status: ${response.status}`, errorBody);
                throw new Error(`Server responded with status ${response.status}`);
            }

            const createdEdges: KartaEdge[] = await response.json();
            return createdEdges;

        } catch (error) {
            console.error(`[ServerAdapter.createEdges] Network error creating edges:`, error);
            throw error;
        }
    }
    async getEdge(edgeId: string): Promise<KartaEdge | undefined> { console.warn(`[ServerAdapter.getEdge] Not implemented for ID: ${edgeId}`); return undefined; }
    async getEdges(): Promise<KartaEdge[]> { console.warn('[ServerAdapter.getEdges] Not implemented'); return []; }

    async reconnectEdge(old_from: NodeId, old_to: NodeId, new_from: NodeId, new_to: NodeId, new_from_path: string, new_to_path: string): Promise<KartaEdge | undefined> {
        const url = `${SERVER_BASE_URL}/api/edges`;
        const payload = {
            old_from,
            old_to,
            new_from,
            new_to,
            new_from_path,
            new_to_path,
        };

        try {
            const response = await fetch(url, {
                method: 'PATCH',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload),
            });

            if (!response.ok) {
                const errorBody = await response.text();
                console.error(`[ServerAdapter.reconnectEdge] Error reconnecting edge. Status: ${response.status}`, errorBody);
                throw new Error(`Server responded with status ${response.status}`);
            }

            const serverEdge: ServerKartaEdge = await response.json();
            return {
                id: serverEdge.uuid,
                source: serverEdge.source,
                target: serverEdge.target,
                attributes: transformServerAttributesToRecord(serverEdge.attributes),
                contains: serverEdge.contains,
            };

        } catch (error) {
            console.error(`[ServerAdapter.reconnectEdge] Network error reconnecting edge:`, error);
            throw error;
        }
    }

    async deleteEdges(payload: EdgeDeletionPayload[]): Promise<void> {
        if (payload.length === 0) {
            return Promise.resolve();
        }

        const url = `${SERVER_BASE_URL}/api/edges`;
        try {
            const response = await fetch(url, {
                method: 'DELETE',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(payload),
            });

            if (!response.ok) {
                const errorBody = await response.text();
                console.error(`[ServerAdapter.deleteEdges] Error deleting edges. Status: ${response.status}`, errorBody);
                throw new Error(`Server responded with status ${response.status}`);
            }

            console.log(`[ServerAdapter.deleteEdges] Successfully requested deletion of ${payload.length} edges.`);

        } catch (error) {
            console.error(`[ServerAdapter.deleteEdges] Network error deleting edges:`, error);
            throw error;
        }
    }



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

    async moveNodes(moves: MoveOperation[]): Promise<MoveNodesResponse> {
        const url = `${SERVER_BASE_URL}/api/nodes/move`;
        const payload = { moves };

        try {
            const response = await fetch(url, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload),
            });

            if (!response.ok) {
                const errorBody = await response.text();
                console.error(`[ServerAdapter.moveNodes] Error moving nodes. Status: ${response.status}`, errorBody);
                throw new Error(`Server responded with status ${response.status}`);
            }

            const result: MoveNodesResponse = await response.json();
            console.log(`[ServerAdapter.moveNodes] Successfully moved ${result.moved_nodes?.length || 0} nodes.`);
            
            // Update all affected nodes in the store
            if (result.moved_nodes && result.moved_nodes.length > 0) {
                const { nodes } = await import('$lib/karta/NodeStore');
                nodes.update(nodeMap => {
                    result.moved_nodes.forEach((movedNodeInfo: any) => {
                        const nodeId = movedNodeInfo.uuid;
                        const newPath = movedNodeInfo.path;
                        const existingNode = nodeMap.get(nodeId);
                        
                        if (existingNode) {
                            const updatedNode = {
                                ...existingNode,
                                path: newPath,
                                attributes: {
                                    ...existingNode.attributes,
                                    name: newPath.split('/').pop() || 'root'
                                }
                            };
                            nodeMap.set(nodeId, updatedNode);
                        }
                    });
                    return nodeMap;
                });
                
                console.log(`[ServerAdapter.moveNodes] Updated ${result.moved_nodes.length} affected nodes in store`);
            }
            
            if (result.errors && result.errors.length > 0) {
                console.warn(`[ServerAdapter.moveNodes] Some operations failed:`, result.errors);
            }
            
            return result;
        } catch (error) {
            console.error(`[ServerAdapter.moveNodes] Network error moving nodes:`, error);
            throw error;
        }
    }

    /**
     * Search for nodes using the backend search API
     * @param query Search query with term, limit, and optional min_score
     * @returns Search response with results, metadata, and timing
     */
    async searchNodes(query: SearchQuery): Promise<SearchResponse> {
        const params = new URLSearchParams();
        params.set('q', query.q);
        
        if (query.limit !== undefined) {
            params.set('limit', query.limit.toString());
        }
        
        if (query.min_score !== undefined) {
            params.set('min_score', query.min_score.toString());
        }

        const url = `${SERVER_BASE_URL}/api/search?${params.toString()}`;
        
        try {
            apiLogger.log(`[ServerAdapter.searchNodes] Searching with query: "${query.q}"`);
            
            const response = await fetch(url);
            
            if (!response.ok) {
                console.error(`[ServerAdapter.searchNodes] Search failed. Status: ${response.status} ${response.statusText}`);
                const errorBody = await response.text();
                console.error(`[ServerAdapter.searchNodes] Error body: ${errorBody}`);
                throw new Error(`Search failed: ${response.status} ${response.statusText}`);
            }
            
            const searchResponse: SearchResponse = await response.json();
            
            apiLogger.log(`[ServerAdapter.searchNodes] Found ${searchResponse.results.length} results in ${searchResponse.took_ms}ms`);
            
            return searchResponse;
        } catch (error) {
            console.error(`[ServerAdapter.searchNodes] Network error during search:`, error);
            throw error;
        }
    }

    /**
     * Get detailed tree structure for export bundle
     */
    async getBundleTree(nodeIds: string[]): Promise<BundleTreeResponse> {
        try {
            const url = `${SERVER_BASE_URL}/api/exports/tree`;
            const params = new URLSearchParams();
            nodeIds.forEach(id => params.append('node_ids', id));
            
            const response = await fetch(`${url}?${params.toString()}`);
            
            if (!response.ok) {
                throw new Error(`Failed to get bundle tree: ${response.status} ${response.statusText}`);
            }
            
            const treeResponse: BundleTreeResponse = await response.json();
            
            apiLogger.log(`[ServerAdapter.getBundleTree] Got tree with ${treeResponse.total_files} files`);
            
            return treeResponse;
        } catch (error) {
            console.error(`[ServerAdapter.getBundleTree] Network error:`, error);
            throw error;
        }
    }

    /**
     * Export nodes as a downloadable bundle
     */
    async exportBundle(request: ExportBundleRequest): Promise<ExportBundleResponse> {
        try {
            const url = `${SERVER_BASE_URL}/api/exports/bundle`;
            
            const response = await fetch(url, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(request)
            });
            
            if (!response.ok) {
                throw new Error(`Failed to export bundle: ${response.status} ${response.statusText}`);
            }
            
            const exportResponse: ExportBundleResponse = await response.json();
            
            apiLogger.log(`[ServerAdapter.exportBundle] Created bundle: ${exportResponse.filename}`);
            
            return exportResponse;
        } catch (error) {
            console.error(`[ServerAdapter.exportBundle] Network error:`, error);
            throw error;
        }
    }

    /**
     * Download a bundle file
     */
    downloadBundle(bundleId: string): string {
        return `${SERVER_BASE_URL}/api/exports/download/${bundleId}`;
    }
}