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
    saveNode(node: DataNode): Promise<void>;
    getNode(nodeId: string): Promise<DataNode | undefined>; // Use DataNode
    deleteNode(nodeId: string): Promise<void>;
    getNodes(): Promise<DataNode[]>; // Add getNodes signature using DataNode

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

        // Directly open the latest DB version (2)
        // The upgrade callback handles creation for all versions sequentially.
        this.dbPromise = idb.openDB<KartaDB>('karta-db', 2, {
            upgrade(db, oldVersion, newVersion, tx) {
                console.log(`[LocalAdapter] Upgrading DB from v${oldVersion} to v${newVersion}...`);
                // Create initial stores if upgrading from v0
                if (oldVersion < 1) {
                    if (!db.objectStoreNames.contains('nodes')) {
                        console.log('[LocalAdapter] Creating "nodes" object store.');
                        db.createObjectStore('nodes', { keyPath: 'id' });
                    }
                    if (!db.objectStoreNames.contains('edges')) {
                        console.log('[LocalAdapter] Creating "edges" object store.');
                        db.createObjectStore('edges', { keyPath: 'id' });
                    }
                }
                // Create contexts store if upgrading from v0 or v1
                if (oldVersion < 2) {
                    if (!db.objectStoreNames.contains('contexts')) {
                        console.log('[LocalAdapter] Creating "contexts" object store.');
                        db.createObjectStore('contexts', { keyPath: 'id' });
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
        return db.getAll('nodes') as Promise<DataNode[]>; // Use DataNode
    }

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

    // --- Context Methods ---

    /**
     * Saves a context, converting absolute ViewNode transforms to relative for storage.
     * Requires the absolute transform of the context's focal node.
     */
    async saveContext(context: Context, focalAbsTransform: AbsoluteTransform = ROOT_TRANSFORM): Promise<void> {
        const db = await this.dbPromise;
        const tx = db.transaction('contexts', 'readwrite');
        const store = tx.objectStore('contexts');

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
                const relScale = viewNode.scale / focalAbsTransform.scale;
                const relRotation = viewNode.rotation - focalAbsTransform.rotation;
                const relX = viewNode.x - focalAbsTransform.x;
                const relY = viewNode.y - focalAbsTransform.y;

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
        value: DataNode; // Use DataNode
    };
    edges: {
        key: string;
        value: KartaEdge;
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
