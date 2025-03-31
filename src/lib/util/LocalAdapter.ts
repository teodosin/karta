import * as idb from 'idb';
// Import Context, ViewNode, NodeId, and AbsoluteTransform
import type { DataNode, KartaEdge, Context, ViewNode, NodeId, AbsoluteTransform } from '../types/types';

// Interface for the ViewNode structure as stored (relative coordinates)
interface StorableViewNode {
    id: NodeId;
    relX: number;
    relY: number;
    width: number;
    height: number;
    relScale: number;
    relRotation: number;
}

// Interface for the storable version of Context (using StorableViewNode)
interface StorableContext {
    id: NodeId;
    viewNodes: [NodeId, StorableViewNode][]; // Store relative ViewNodes
}

// Define default transform for root context
const ROOT_TRANSFORM: AbsoluteTransform = { x: 0, y: 0, scale: 1, rotation: 0 };

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

    // Add Context methods - now require focal transform for conversion
    saveContext(context: Context, focalAbsTransform?: AbsoluteTransform): Promise<void>;
    getContext(contextId: NodeId, focalAbsTransform?: AbsoluteTransform): Promise<Context | undefined>;
    getContexts(focalAbsTransforms: Map<NodeId, AbsoluteTransform>): Promise<Context[]>; // Needs transforms for all contexts
}

class LocalAdapter implements PersistenceService {
    private dbPromise: Promise<idb.IDBPDatabase<KartaDB>>;

    constructor() {
        console.log('[LocalAdapter] Constructor: Initializing DB connection...');
        const startTime = performance.now();

        // Increment DB version to 4 to trigger the upgrade for the new edge indexes
        // The upgrade callback handles creation for all versions sequentially.
        this.dbPromise = idb.openDB<KartaDB>('karta-db', 4, {
            upgrade(db, oldVersion, newVersion, tx, event) {
                console.log(`[LocalAdapter] Upgrading DB from v${oldVersion} to v${newVersion}...`);
                // Create initial stores if upgrading from v0
                // Create initial stores if upgrading from v0
                if (oldVersion < 1) {
                    if (!db.objectStoreNames.contains('nodes')) {
                        console.log('[LocalAdapter] Creating "nodes" object store.');
                        const nodeStore = db.createObjectStore('nodes', { keyPath: 'id' });
                        // Add index for name checking (non-unique)
                        console.log('[LocalAdapter] Creating "name_idx" index on "nodes" store.');
                        nodeStore.createIndex('name_idx', 'attributes.name', { unique: false });
                    }
                    if (!db.objectStoreNames.contains('edges')) {
                        console.log('[LocalAdapter] Creating "edges" object store.');
                        const edgeStore = db.createObjectStore('edges', { keyPath: 'id' });
                        // Add indexes for source and target
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
                 // Add name index if upgrading from v1 or v2 (where it didn't exist)
                 // Note: Accessing transaction directly via 'tx' might be unreliable across versions/libraries.
                 // It's safer to check within the upgrade logic if the index needs creation.
                 // However, the idb library handles this well: if createIndex is called and it exists, it's a no-op.
                 // Add name index if upgrading from v1 or v2 (where it didn't exist)
                 if (oldVersion >= 1 && oldVersion < 3) {
                    // Ensure the nodes store exists before trying to add an index
                    if (db.objectStoreNames.contains('nodes')) {
                        // The transaction is automatically available on the db object within the upgrade callback
                        const nodeStore = tx.objectStore('nodes');
                        if (!nodeStore.indexNames.contains('name_idx')) {
                            console.log('[LocalAdapter] Creating "name_idx" index on existing "nodes" store.');
                            nodeStore.createIndex('name_idx', 'attributes.name', { unique: false });
                        }
                    } else {
                         console.warn('[LocalAdapter] Cannot add name_idx: "nodes" store does not exist.');
                    }
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
                    } else {
                        console.warn('[LocalAdapter] Cannot add edge indexes: "edges" store does not exist.');
                    }
                 }
                console.log('[LocalAdapter] DB upgrade complete.');
            },
            blocked() {
                // Log error instead of alerting the user
                console.error('[LocalAdapter] IDB blocked. Close other tabs accessing the DB.');
                // alert('Database access is blocked. Please close other tabs/windows accessing this application and refresh.'); // Removed alert
            },
            blocking() {
                    console.warn('[LocalAdapter] IDB blocking. Connection will close.');
                    // db.close(); // Close the connection when blocking others
                },
                terminated() {
                     console.warn('[LocalAdapter] IDB connection terminated unexpectedly.');
                }
            });


        this.dbPromise.then(db => {
            const endTime = performance.now();
            console.log(`[LocalAdapter] DB connection established successfully in ${endTime - startTime}ms. DB Name: ${db.name}, Version: ${db.version}`);
        }).catch(error => {
             const endTime = performance.now();
            console.error(`[LocalAdapter] DB connection failed after ${endTime - startTime}ms:`, error);
        });
    }

    async saveNode(node: DataNode): Promise<void> { // Use DataNode
        const db = await this.dbPromise;
        const tx = db.transaction('nodes', 'readwrite');
        const store = tx.objectStore('nodes');
        await store.put(node);
        await tx.done;
    }

    async getNode(nodeId: string): Promise<DataNode | undefined> { // Use DataNode
        const db = await this.dbPromise;
        return db.get('nodes', nodeId) as Promise<DataNode | undefined>; // Use DataNode
    }

    async deleteNode(nodeId: string): Promise<void> {
        const db = await this.dbPromise;
        const tx = db.transaction('nodes', 'readwrite');
        const store = tx.objectStore('nodes');
        await store.delete(nodeId);
        await tx.done;
    }

    async getNodes(): Promise<DataNode[]> { // Use DataNode
        const db = await this.dbPromise;
        return db.getAll('nodes') as Promise<DataNode[]>;
    }

    /**
     * Checks if a node with the given name exists using the name index.
     */
    async checkNameExists(name: string): Promise<boolean> {
        const db = await this.dbPromise;
        const tx = db.transaction('nodes', 'readonly');
        const store = tx.objectStore('nodes');
        const index = store.index('name_idx');
        const count = await index.count(name); // Count items matching the name
        await tx.done;
        return count > 0;
    }

     /**
     * Gets multiple DataNodes by their IDs.
     */
    async getDataNodesByIds(nodeIds: NodeId[]): Promise<Map<NodeId, DataNode>> {
        const db = await this.dbPromise;
        const tx = db.transaction('nodes', 'readonly');
        const store = tx.objectStore('nodes');
        const nodesMap = new Map<NodeId, DataNode>();

        // Fetch nodes concurrently
        await Promise.all(nodeIds.map(async (id) => {
            const node = await store.get(id);
            if (node) {
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
        const store = tx.objectStore('edges');
        await store.put(edge);
        await tx.done;
    }

    async getEdge(edgeId: string): Promise<KartaEdge | undefined> {
        const db = await this.dbPromise;
        return db.get('edges', edgeId) as Promise<KartaEdge | undefined>;
    }

    async getEdges(): Promise<KartaEdge[]> {
        const db = await this.dbPromise;
        return db.getAll('edges') as Promise<KartaEdge[]>;
    }

    async deleteEdge(edgeId: string): Promise<void> {
        const db = await this.dbPromise;
        const tx = db.transaction('edges', 'readwrite');
        const store = tx.objectStore('edges');
        await store.delete(edgeId);
        await tx.done;
    }

    async loadEdges(): Promise<KartaEdge[]> { // Add loadEdges method
        const db = await this.dbPromise;
        return db.getAll('edges') as Promise<KartaEdge[]>;
    }

    /**
     * Gets all edges connected to a given set of node IDs using source and target indexes.
     */
    async getEdgesByNodeIds(nodeIds: NodeId[]): Promise<Map<string, KartaEdge>> {
        if (nodeIds.length === 0) {
            return new Map();
        }
        const db = await this.dbPromise;
        const tx = db.transaction('edges', 'readonly');
        const store = tx.objectStore('edges');
        const sourceIndex = store.index('source_idx');
        const targetIndex = store.index('target_idx');
        const relevantEdges = new Map<string, KartaEdge>();

        // Fetch edges where source matches any nodeId
        const sourcePromises = nodeIds.map(id => sourceIndex.getAll(id));
        // Fetch edges where target matches any nodeId
        const targetPromises = nodeIds.map(id => targetIndex.getAll(id));

        // Wait for all queries to complete
        const [sourceResults, targetResults] = await Promise.all([
            Promise.all(sourcePromises),
            Promise.all(targetPromises)
        ]);

        // Combine results, using Map to automatically handle duplicates
        sourceResults.flat().forEach(edge => relevantEdges.set(edge.id, edge));
        targetResults.flat().forEach(edge => relevantEdges.set(edge.id, edge));

        await tx.done;
        return relevantEdges;
    }


    // --- Context Methods ---

    /**
     * Saves a context, converting absolute ViewNode transforms to relative for storage.
     * Requires the absolute transform of the context's focal node.
     */
    async saveContext(context: Context): Promise<void> {
        const db = await this.dbPromise;
        const tx = db.transaction('contexts', 'readwrite');
        const store = tx.objectStore('contexts');

        const focalNode = context.viewNodes.get(context.id);
        if (!focalNode) throw new Error("Focal node not found in context");

        // Convert absolute ViewNodes to relative StorableViewNodes
        const storableViewNodes: [NodeId, StorableViewNode][] = [];
        for (const [nodeId, viewNode] of context.viewNodes.entries()) {
            let storableNode: StorableViewNode;
            if (nodeId === context.id) {
                // Focal node always has relative transform of origin/identity when saved in its own context
                storableNode = {
                    id: viewNode.id,
                    relX: 0,
                    relY: 0,
                    width: viewNode.width,
                    height: viewNode.height,
                    relScale: 1,
                    relRotation: 0
                };
            } else {
                // Calculate relative transform for child nodes
                const relScale = viewNode.scale / focalNode.scale;
                const relRotation = viewNode.rotation - focalNode.rotation;
                const relX = viewNode.x - focalNode.x;
                const relY = viewNode.y - focalNode.y;

                storableNode = {
                    id: viewNode.id,
                    relX,
                    relY,
                    width: viewNode.width,
                    height: viewNode.height,
                    relScale,
                    relRotation
                };
            }
            storableViewNodes.push([nodeId, storableNode]);
        }

        const storableContext: StorableContext = {
            id: context.id,
            viewNodes: storableViewNodes,
        };
        await store.put(storableContext);
        await tx.done;
    }

    /**
     * Gets a single context, converting stored relative transforms to absolute.
     * Requires the absolute transform of the context's focal node.
     */
    async getContext(contextId: NodeId, focalAbsTransform: AbsoluteTransform = ROOT_TRANSFORM): Promise<Context | undefined> {
        const db = await this.dbPromise;
        const storableContext = await db.get('contexts', contextId) as StorableContext | undefined;

        if (storableContext) {
            // Convert relative StorableViewNodes back to absolute ViewNodes
            const absoluteViewNodes = new Map<NodeId, ViewNode>();
            for (const [nodeId, storableViewNode] of storableContext.viewNodes) {
                let absoluteNode: ViewNode;
                if (nodeId === contextId) {
                    // Focal node's absolute transform is the one provided
                    absoluteNode = {
                        id: storableViewNode.id,
                        x: focalAbsTransform.x,
                        y: focalAbsTransform.y,
                        width: storableViewNode.width,
                        height: storableViewNode.height,
                        scale: focalAbsTransform.scale,
                        rotation: focalAbsTransform.rotation
                    };
                } else {
                    // Calculate absolute transform for child nodes
                    const absScale = focalAbsTransform.scale * storableViewNode.relScale;
                    const absRotation = focalAbsTransform.rotation + storableViewNode.relRotation;
                    const scaledRelX = storableViewNode.relX * focalAbsTransform.scale;
                    const scaledRelY = storableViewNode.relY * focalAbsTransform.scale;
                    const angleRad = (focalAbsTransform.rotation * Math.PI) / 180;
                    const cosAngle = Math.cos(angleRad);
                    const sinAngle = Math.sin(angleRad);
                    const rotatedScaledRelX = scaledRelX * cosAngle - scaledRelY * sinAngle;
                    const rotatedScaledRelY = scaledRelX * sinAngle + scaledRelY * cosAngle;
                    const absX = focalAbsTransform.x + rotatedScaledRelX;
                    const absY = focalAbsTransform.y + rotatedScaledRelY;

                    absoluteNode = {
                        id: storableViewNode.id,
                        x: absX,
                        y: absY,
                        width: storableViewNode.width,
                        height: storableViewNode.height,
                        scale: absScale,
                        rotation: absRotation
                    };
                }
                absoluteViewNodes.set(nodeId, absoluteNode);
            }

            return {
                id: storableContext.id,
                viewNodes: absoluteViewNodes,
            };
        }
        return undefined;
    }

     /**
     * Gets all contexts, converting stored relative transforms to absolute.
     * Requires a map providing the absolute transform for each context's focal node.
     */
    async getContexts(focalAbsTransforms: Map<NodeId, AbsoluteTransform>): Promise<Context[]> {
        const db = await this.dbPromise;
        const storableContexts = await db.getAll('contexts') as StorableContext[];

        // Convert relative StorableViewNodes back to absolute ViewNodes for each context
        return storableContexts.map(storable => {
            const focalAbsTransform = focalAbsTransforms.get(storable.id) ?? ROOT_TRANSFORM; // Use root if not found (shouldn't happen ideally)
            const absoluteViewNodes = new Map<NodeId, ViewNode>();

            for (const [nodeId, storableViewNode] of storable.viewNodes) {
                let absoluteNode: ViewNode;
                 if (nodeId === storable.id) {
                    // Focal node's absolute transform is the one provided
                    absoluteNode = {
                        id: storableViewNode.id,
                        x: focalAbsTransform.x,
                        y: focalAbsTransform.y,
                        width: storableViewNode.width,
                        height: storableViewNode.height,
                        scale: focalAbsTransform.scale,
                        rotation: focalAbsTransform.rotation
                    };
                 } else {
                    // Calculate absolute transform for child nodes
                    const absScale = focalAbsTransform.scale * storableViewNode.relScale;
                    const absRotation = focalAbsTransform.rotation + storableViewNode.relRotation;
                    const scaledRelX = storableViewNode.relX * focalAbsTransform.scale;
                    const scaledRelY = storableViewNode.relY * focalAbsTransform.scale;
                    const angleRad = (focalAbsTransform.rotation * Math.PI) / 180;
                    const cosAngle = Math.cos(angleRad);
                    const sinAngle = Math.sin(angleRad);
                    const rotatedScaledRelX = scaledRelX * cosAngle - scaledRelY * sinAngle;
                    const rotatedScaledRelY = scaledRelX * sinAngle + scaledRelY * cosAngle;
                    const absX = focalAbsTransform.x + rotatedScaledRelX;
                    const absY = focalAbsTransform.y + rotatedScaledRelY;

                    absoluteNode = {
                        id: storableViewNode.id,
                        x: absX,
                        y: absY,
                        width: storableViewNode.width,
                        height: storableViewNode.height,
                        scale: absScale,
                        rotation: absRotation
                    };
                 }
                absoluteViewNodes.set(nodeId, absoluteNode);
            }
            return {
                id: storable.id,
                viewNodes: absoluteViewNodes,
            };
        });
    }

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
    contexts: { // Add contexts store definition
        key: NodeId;
        value: StorableContext; // Stores relative StorableViewNodes
    };
}


let localAdapterInstance: LocalAdapter | null = null;

if (typeof window !== 'undefined') {
    localAdapterInstance = new LocalAdapter();
}

export const localAdapter = localAdapterInstance;
