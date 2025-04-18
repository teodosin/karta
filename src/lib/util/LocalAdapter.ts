import * as idb from 'idb';
// Import Context, ViewNode, NodeId, and AbsoluteTransform
// Also import ViewportSettings
import type { DataNode, KartaEdge, Context, ViewNode, NodeId, AbsoluteTransform, ViewportSettings, StorableContext, StorableViewNode, StorableViewportSettings, AssetData } from '../types/types'; // Import Storable types & AssetData

// Removed local definitions for StorableViewNode, StorableViewportSettings, StorableContext
// They are now imported from types.ts

// Define default transform for root context
const ROOT_TRANSFORM: AbsoluteTransform = { x: 0, y: 0, scale: 1 }; // Rotation removed

interface PersistenceService {
    // Node methods
    saveNode(node: DataNode): Promise<void>;
    getNode(nodeId: string): Promise<DataNode | undefined>; // Use DataNode
    deleteNode(nodeId: string): Promise<void>;
    getNodes(): Promise<DataNode[]>;
    checkNameExists(name: string): Promise<boolean>; // Add checkNameExists signature

    // Edge methods
    saveEdge(edge: KartaEdge): Promise<void>;
    getEdge(edgeId: string): Promise<KartaEdge | undefined>;
    getEdges(): Promise<KartaEdge[]>;
    deleteEdge(edgeId: string): Promise<void>;

    // Context methods (no longer pass focalAbsTransform to saveContext)
    // saveContext still takes the in-memory Context (with Tweens)
    saveContext(context: Context): Promise<void>;
    // getContext returns the StorableContext read from DB
    getContext(contextId: NodeId): Promise<StorableContext | undefined>;
    // getContexts(focalAbsTransforms: Map<NodeId, AbsoluteTransform>): Promise<Context[]>; // Removed unused function
}

class LocalAdapter implements PersistenceService {
    private dbPromise: Promise<idb.IDBPDatabase<KartaDB>>;
    private objectUrlMap = new Map<string, string>(); // Tracks generated Object URLs { nodeId: objectUrl }

    constructor() {
        console.log('[LocalAdapter] Constructor: Initializing DB connection...');
        const startTime = performance.now();

        // DB Version 5: Added assets store
        this.dbPromise = idb.openDB<KartaDB>('karta-db', 5, {
            upgrade(db, oldVersion, newVersion, tx, event) {
                console.log(`[LocalAdapter] Upgrading DB from v${oldVersion} to v${newVersion}...`);
                // Create initial stores if upgrading from v0
                if (oldVersion < 1) {
                    if (!db.objectStoreNames.contains('nodes')) {
                        console.log('[LocalAdapter] Creating "nodes" object store.');
                        const nodeStore = db.createObjectStore('nodes', { keyPath: 'id' });
                        console.log('[LocalAdapter] Creating "name_idx" index on "nodes" store.');
                        nodeStore.createIndex('name_idx', 'attributes.name', { unique: false });
                    }
                    if (!db.objectStoreNames.contains('edges')) {
                        console.log('[LocalAdapter] Creating "edges" object store.');
                        const edgeStore = db.createObjectStore('edges', { keyPath: 'id' });
                        console.log('[LocalAdapter] Creating "source_idx" and "target_idx" indexes on "edges" store.');
                        edgeStore.createIndex('source_idx', 'source', { unique: false });
                        edgeStore.createIndex('target_idx', 'target', { unique: false });
                    }
                }
                // Create contexts store if upgrading from v0 or v1
                if (oldVersion < 2) {
                    if (!db.objectStoreNames.contains('contexts')) {
                        console.log('[LocalAdapter] Creating "contexts" object store.');
                        db.createObjectStore('contexts', { keyPath: 'id' });
                    }
                }
                 // Add name index if upgrading from v1 or v2
                 if (oldVersion >= 1 && oldVersion < 3) {
                    if (db.objectStoreNames.contains('nodes')) {
                        const nodeStore = tx.objectStore('nodes');
                        if (!nodeStore.indexNames.contains('name_idx')) {
                            console.log('[LocalAdapter] Creating "name_idx" index on existing "nodes" store.');
                            nodeStore.createIndex('name_idx', 'attributes.name', { unique: false });
                        }
                    } else { console.warn('[LocalAdapter] Cannot add name_idx: "nodes" store does not exist.'); }
                 }
                 // Add edge indexes if upgrading from v1, v2, or v3
                 if (oldVersion >= 1 && oldVersion < 4) {
                    if (db.objectStoreNames.contains('edges')) {
                        const edgeStore = tx.objectStore('edges');
                        if (!edgeStore.indexNames.contains('source_idx')) {
                            console.log('[LocalAdapter] Creating "source_idx" index on existing "edges" store.');
                            edgeStore.createIndex('source_idx', 'source', { unique: false });
                        }
                        if (!edgeStore.indexNames.contains('target_idx')) {
                            console.log('[LocalAdapter] Creating "target_idx" index on existing "edges" store.');
                            edgeStore.createIndex('target_idx', 'target', { unique: false });
                        }
                    } else { console.warn('[LocalAdapter] Cannot add edge indexes: "edges" store does not exist.'); }
                 }
                // Create assets store if upgrading from v0, v1, v2, v3, or v4
                if (oldVersion < 5) {
                   if (!db.objectStoreNames.contains('assets')) {
                       console.log('[LocalAdapter] Creating "assets" object store.');
                       db.createObjectStore('assets'); // Key is assetId (usually nodeId)
                   }
                }
                console.log('[LocalAdapter] DB upgrade complete.');
            },
            blocked() { console.error('[LocalAdapter] IDB blocked. Close other tabs accessing the DB.'); },
            blocking() { console.warn('[LocalAdapter] IDB blocking. Connection will close.'); },
            terminated() { console.warn('[LocalAdapter] IDB connection terminated unexpectedly.'); }
            });

        this.dbPromise.then(db => {
            const endTime = performance.now();
            console.log(`[LocalAdapter] DB connection established successfully in ${endTime - startTime}ms. DB Name: ${db.name}, Version: ${db.version}`);
        }).catch(error => {
             const endTime = performance.now();
            console.error(`[LocalAdapter] DB connection failed after ${endTime - startTime}ms:`, error);
        });
    }

    async saveNode(node: DataNode): Promise<void> {
        const db = await this.dbPromise;
        const tx = db.transaction('nodes', 'readwrite');
        await tx.objectStore('nodes').put(node);
        await tx.done;
    }

    async getNode(nodeId: string): Promise<DataNode | undefined> {
        const db = await this.dbPromise;
        const node = await db.get('nodes', nodeId);
        if (node && node.ntype === 'image' && node.attributes.assetId) {
            await this.updateNodeSrcWithObjectUrl(node);
        }
        return node;
    }

    async deleteNode(nodeId: string): Promise<void> {
        const db = await this.dbPromise;
        const tx = db.transaction('nodes', 'readwrite');
        await tx.objectStore('nodes').delete(nodeId);
        await tx.done;
    }

    async getNodes(): Promise<DataNode[]> {
        const db = await this.dbPromise;
        const nodes = await db.getAll('nodes');
        // Generate Object URLs for image nodes after loading
        await Promise.all(nodes.map(node => {
            if (node.ntype === 'image' && node.attributes.assetId) {
                return this.updateNodeSrcWithObjectUrl(node);
            }
            return Promise.resolve();
        }));
        return nodes;
    }

    async checkNameExists(name: string): Promise<boolean> {
        const db = await this.dbPromise;
        const tx = db.transaction('nodes', 'readonly');
        const index = tx.objectStore('nodes').index('name_idx');
        const count = await index.count(name);
        await tx.done;
        return count > 0;
    }

    async getDataNodesByIds(nodeIds: NodeId[]): Promise<Map<NodeId, DataNode>> {
        const db = await this.dbPromise;
        const tx = db.transaction('nodes', 'readonly');
        const store = tx.objectStore('nodes');
        const nodesMap = new Map<NodeId, DataNode>();
        await Promise.all(nodeIds.map(async (id) => {
            const node = await store.get(id);
            if (node) {
                 if (node.ntype === 'image' && node.attributes.assetId) {
                    // Generate Object URL before adding to map
                    await this.updateNodeSrcWithObjectUrl(node);
                 }
                 nodesMap.set(id, node);
            }
        }));
        await tx.done;
        return nodesMap;
    }

    // --- Edge Methods ---

    async saveEdge(edge: KartaEdge): Promise<void> {
        const db = await this.dbPromise;
        const tx = db.transaction('edges', 'readwrite');
        await tx.objectStore('edges').put(edge);
        await tx.done;
    }

    async getEdge(edgeId: string): Promise<KartaEdge | undefined> {
        const db = await this.dbPromise;
        return db.get('edges', edgeId);
    }

    async getEdges(): Promise<KartaEdge[]> {
        const db = await this.dbPromise;
        return db.getAll('edges');
    }

    async deleteEdge(edgeId: string): Promise<void> {
        const db = await this.dbPromise;
        const tx = db.transaction('edges', 'readwrite');
        await tx.objectStore('edges').delete(edgeId);
        await tx.done;
    }

    async loadEdges(): Promise<KartaEdge[]> {
        const db = await this.dbPromise;
        return db.getAll('edges');
    }

    async getEdgesByNodeIds(nodeIds: NodeId[]): Promise<Map<string, KartaEdge>> {
        if (nodeIds.length === 0) return new Map();
        const db = await this.dbPromise;
        const tx = db.transaction('edges', 'readonly');
        const store = tx.objectStore('edges');
        const sourceIndex = store.index('source_idx');
        const targetIndex = store.index('target_idx');
        const relevantEdges = new Map<string, KartaEdge>();
        const sourcePromises = nodeIds.map(id => sourceIndex.getAll(id));
        const targetPromises = nodeIds.map(id => targetIndex.getAll(id));
        const [sourceResults, targetResults] = await Promise.all([
            Promise.all(sourcePromises), Promise.all(targetPromises)
        ]);
        sourceResults.flat().forEach(edge => relevantEdges.set(edge.id, edge));
        targetResults.flat().forEach(edge => relevantEdges.set(edge.id, edge));
        await tx.done;
        return relevantEdges;
    }

    // --- Asset Methods ---

    async saveAsset(assetId: string, assetData: AssetData): Promise<void> {
        const db = await this.dbPromise;
        const tx = db.transaction('assets', 'readwrite');
        await tx.objectStore('assets').put(assetData, assetId);
        await tx.done;
        console.log(`[LocalAdapter] Saved asset ${assetId}`);
    }

    async getAsset(assetId: string): Promise<AssetData | undefined> {
        const db = await this.dbPromise;
        return db.get('assets', assetId);
    }

    async deleteAsset(assetId: string): Promise<void> {
        const db = await this.dbPromise;
        const tx = db.transaction('assets', 'readwrite');
        await tx.objectStore('assets').delete(assetId);
        await tx.done;
        // Revoke and remove Object URL if it exists for this assetId (which is usually nodeId)
        const existingUrl = this.objectUrlMap.get(assetId);
        if (existingUrl) {
            URL.revokeObjectURL(existingUrl);
            this.objectUrlMap.delete(assetId);
            console.log(`[LocalAdapter] Revoked and removed Object URL for asset ${assetId}`);
        }
         console.log(`[LocalAdapter] Deleted asset ${assetId}`);
    }

    // Helper to update a node's src attribute with a generated Object URL
    private async updateNodeSrcWithObjectUrl(node: DataNode): Promise<void> {
        if (!node.attributes.assetId) return; // Should not happen if called correctly

        const assetId = node.attributes.assetId;
        const assetData = await this.getAsset(assetId);

        // Revoke previous URL for this node if it exists
        const oldUrl = this.objectUrlMap.get(node.id);
        if (oldUrl) {
            URL.revokeObjectURL(oldUrl);
            this.objectUrlMap.delete(node.id);
        }

        if (assetData?.blob) {
            try {
                const newObjectUrl = URL.createObjectURL(assetData.blob);
                node.attributes.src = newObjectUrl; // Update the node object directly
                this.objectUrlMap.set(node.id, newObjectUrl); // Track the new URL
                // console.log(`[LocalAdapter] Generated Object URL for node ${node.id}: ${newObjectUrl}`);
            } catch (error) {
                 console.error(`[LocalAdapter] Error creating Object URL for node ${node.id}:`, error);
                 node.attributes.src = ''; // Set src to empty on error
            }
        } else {
            console.warn(`[LocalAdapter] Asset data or blob not found for assetId ${assetId} (node ${node.id}). Setting src to empty.`);
            node.attributes.src = ''; // Set src to empty if asset is missing
        }
    }

    // Method to clean up all generated Object URLs
    // Use arrow function syntax to ensure 'this' context is correct when used as event listener
    private cleanupObjectUrls = (): void => {
        console.log(`[LocalAdapter] Cleaning up ${this.objectUrlMap.size} Object URLs before unload...`);
        this.objectUrlMap.forEach((url, nodeId) => {
            URL.revokeObjectURL(url);
            // console.log(`[LocalAdapter] Revoked Object URL for node ${nodeId}`);
        });
        this.objectUrlMap.clear();
        console.log('[LocalAdapter] Object URL map cleared.');
    };

    // --- Context Methods ---

    async saveContext(context: Context): Promise<void> {
        const db = await this.dbPromise;
        const tx = db.transaction('contexts', 'readwrite');
        const store = tx.objectStore('contexts');
        const focalNode = context.viewNodes.get(context.id);
        // Access focal node state via the tween
        const focalNodeState = focalNode?.state.current;
        console.log("focalNode position", focalNodeState?.x, focalNodeState?.y);
        if (!focalNodeState) throw new Error(`Focal node ${context.id} state not found in context being saved`);

        // Convert ViewNodes (containing Tweens) to StorableViewNodes (relative positions)
        const storableViewNodes: [NodeId, StorableViewNode][] = [];
        for (const [nodeId, viewNode] of context.viewNodes.entries()) {
            const nodeState = viewNode.state.current; // Get current state from tween
            let storableNode: StorableViewNode;
            if (nodeId === context.id) {
                // Focal node is always relative origin
                storableNode = { id: viewNode.id, relX: 0, relY: 0, width: nodeState.width, height: nodeState.height, relScale: 1, rotation: nodeState.rotation };
            } else {
                // Calculate relative properties based on focal node's current state
                const relScale = nodeState.scale / focalNodeState.scale;
                const dx = nodeState.x - focalNodeState.x;
                const dy = nodeState.y - focalNodeState.y;
                // Simplified relative position calculation (no rotation considered for offset)
                const relX = dx / focalNodeState.scale;
                const relY = dy / focalNodeState.scale;
                storableNode = { id: viewNode.id, relX, relY, width: nodeState.width, height: nodeState.height, relScale, rotation: nodeState.rotation };
            }
            storableViewNodes.push([nodeId, storableNode]);
        }

        // Convert absolute ViewportSettings to relative StorableViewportSettings
        let storableViewportSettings: StorableViewportSettings | undefined = undefined;
        if (context.viewportSettings) {
            const absSettings = context.viewportSettings;
            // Use focal node's current state for calculation
            let dx = (focalNodeState.x * absSettings.scale) + absSettings.posX;
            let dy = (focalNodeState.y * absSettings.scale) + absSettings.posY;
            console.log("absSettings", absSettings);
            console.log("Relative viewport position dx and dy", dx, " ", dy);

            storableViewportSettings = {
                scale: absSettings.scale, // Scale is absolute
                relPosX: dx,
                relPosY: dy
            };
        }

        const storableContext: StorableContext = {
            id: context.id,
            viewNodes: storableViewNodes,
            viewportSettings: storableViewportSettings // Save relative viewport settings
        };
        await store.put(storableContext);
        await tx.done;
    }

    // getContext now returns the raw StorableContext
    // Conversion to in-memory Context (with Tweens) happens in KartaStore
    async getContext(contextId: NodeId): Promise<StorableContext | undefined> {
        const db = await this.dbPromise;
        const storableContext = await db.get('contexts', contextId) as StorableContext | undefined;
        return storableContext;
    }

    // New method to get all context IDs
    async getAllContextIds(): Promise<NodeId[]> {
        try {
            const db = await this.dbPromise;
            const tx = db.transaction('contexts', 'readonly');
            const store = tx.objectStore('contexts');
            const allKeys = await store.getAllKeys();
            await tx.done;
            // Ensure keys are returned as NodeId[] (string[])
            return allKeys as NodeId[];
        } catch (error) {
            console.error("[LocalAdapter] Error getting all context IDs:", error);
            return []; // Return empty array on error
        }
    }

    // getContexts needs similar conversion logic for viewportSettings
    // Removed unused getContexts function
}

// Define database schema (for TypeScript type checking)
interface KartaDB extends idb.DBSchema {
    nodes: {
        key: string;
        value: DataNode;
        indexes: { 'name_idx': string }; // Define the index
    };
    edges: {
        key: string;
        value: KartaEdge;
        indexes: { 'source_idx': string; 'target_idx': string }; // Define edge indexes
    };
    contexts: {
        key: NodeId;
        value: StorableContext; // Stores relative StorableViewNodes and StorableViewportSettings
    };
    assets: { // New store for binary asset data
        key: string; // assetId (usually the nodeId of the image node)
        value: AssetData; // { blob: Blob, mimeType: string, name: string }
    };
}

let localAdapterInstance: LocalAdapter | null = null;
if (typeof window !== 'undefined') {
    localAdapterInstance = new LocalAdapter();
}
export const localAdapter = localAdapterInstance;
