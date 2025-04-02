import * as idb from 'idb';
// Import Context, ViewNode, NodeId, and AbsoluteTransform
// Also import ViewportSettings
import type { DataNode, KartaEdge, Context, ViewNode, NodeId, AbsoluteTransform, ViewportSettings } from '../types/types';

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

// Interface for storing relative viewport settings
interface StorableViewportSettings {
    scale: number;
    relPosX: number; // Relative X
    relPosY: number; // Relative Y
}

// Interface for the storable version of Context (using StorableViewNode)
interface StorableContext {
    id: NodeId;
    viewNodes: [NodeId, StorableViewNode][]; // Store relative ViewNodes
    viewportSettings?: StorableViewportSettings; // Store relative viewport settings
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

    // Context methods (no longer pass focalAbsTransform to saveContext)
    saveContext(context: Context): Promise<void>;
    getContext(contextId: NodeId, focalAbsTransform?: AbsoluteTransform): Promise<Context | undefined>;
    getContexts(focalAbsTransforms: Map<NodeId, AbsoluteTransform>): Promise<Context[]>; // Needs transforms for all contexts
}

class LocalAdapter implements PersistenceService {
    private dbPromise: Promise<idb.IDBPDatabase<KartaDB>>;

    constructor() {
        console.log('[LocalAdapter] Constructor: Initializing DB connection...');
        const startTime = performance.now();

        // DB Version 4: Added edge indexes
        this.dbPromise = idb.openDB<KartaDB>('karta-db', 4, {
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
        return db.get('nodes', nodeId);
    }

    async deleteNode(nodeId: string): Promise<void> {
        const db = await this.dbPromise;
        const tx = db.transaction('nodes', 'readwrite');
        await tx.objectStore('nodes').delete(nodeId);
        await tx.done;
    }

    async getNodes(): Promise<DataNode[]> {
        const db = await this.dbPromise;
        return db.getAll('nodes');
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
            if (node) nodesMap.set(id, node);
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

    // --- Context Methods ---

    async saveContext(context: Context): Promise<void> {
        const db = await this.dbPromise;
        const tx = db.transaction('contexts', 'readwrite');
        const store = tx.objectStore('contexts');
        const focalNode = context.viewNodes.get(context.id);
        if (!focalNode) throw new Error(`Focal node ${context.id} not found in context being saved`);

        // Convert ViewNodes to StorableViewNodes (relative positions)
        const storableViewNodes: [NodeId, StorableViewNode][] = [];
        for (const [nodeId, viewNode] of context.viewNodes.entries()) {
            let storableNode: StorableViewNode;
            if (nodeId === context.id) {
                storableNode = { id: viewNode.id, relX: 0, relY: 0, width: viewNode.width, height: viewNode.height, relScale: 1, relRotation: 0 };
            } else {
                const relScale = viewNode.scale / focalNode.scale;
                const relRotation = viewNode.rotation - focalNode.rotation;
                const dx = viewNode.x - focalNode.x;
                const dy = viewNode.y - focalNode.y;
                const angleRad = (-focalNode.rotation * Math.PI) / 180; // Inverse rotation
                const cosAngle = Math.cos(angleRad);
                const sinAngle = Math.sin(angleRad);
                const relX = (dx * cosAngle - dy * sinAngle) / focalNode.scale; // Rotate and scale back
                const relY = (dx * sinAngle + dy * cosAngle) / focalNode.scale;
                storableNode = { id: viewNode.id, relX, relY, width: viewNode.width, height: viewNode.height, relScale, relRotation };
            }
            // DEBUG: Log conversion for saveContext
            console.log(`[DEBUG saveContext ${context.id}] Node ${nodeId} | AbsIn: ${JSON.stringify({x: viewNode.x, y: viewNode.y, scale: viewNode.scale, rotation: viewNode.rotation})} | RelOut: ${JSON.stringify(storableNode)} | Focal: ${JSON.stringify(focalNode)}`);
            storableViewNodes.push([nodeId, storableNode]);
        }

        // Convert absolute ViewportSettings to relative StorableViewportSettings
        let storableViewportSettings: StorableViewportSettings | undefined = undefined;
        if (context.viewportSettings) {
            const absSettings = context.viewportSettings;
            const dx = absSettings.posX - focalNode.x;
            const dy = absSettings.posY - focalNode.y;
            const angleRad = (-focalNode.rotation * Math.PI) / 180; // Inverse rotation
            const cosAngle = Math.cos(angleRad);
            const sinAngle = Math.sin(angleRad);
            const relPosX = (dx * cosAngle - dy * sinAngle) / focalNode.scale; // Rotate and scale back
            const relPosY = (dx * sinAngle + dy * cosAngle) / focalNode.scale;
            storableViewportSettings = {
                scale: absSettings.scale, // Scale is absolute
                relPosX: relPosX,
                relPosY: relPosY
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

    async getContext(contextId: NodeId, focalAbsTransform: AbsoluteTransform = ROOT_TRANSFORM): Promise<Context | undefined> {
        const db = await this.dbPromise;
        const storableContext = await db.get('contexts', contextId) as StorableContext | undefined;

        if (storableContext) {
            // Convert StorableViewNodes to ViewNodes (absolute positions)
            const absoluteViewNodes = new Map<NodeId, ViewNode>();
            for (const [nodeId, storableViewNode] of storableContext.viewNodes) {
                let absoluteNode: ViewNode;
                if (nodeId === contextId) {
                    absoluteNode = {
                        id: storableViewNode.id, x: focalAbsTransform.x, y: focalAbsTransform.y,
                        width: storableViewNode.width, height: storableViewNode.height,
                        scale: focalAbsTransform.scale, rotation: focalAbsTransform.rotation
                    };
                } else {
                    const absScale = focalAbsTransform.scale * storableViewNode.relScale;
                    const absRotation = focalAbsTransform.rotation + storableViewNode.relRotation;
                    // Calculate absolute position correctly
                    const scaledRelX = storableViewNode.relX * focalAbsTransform.scale; // Scale first
                    const scaledRelY = storableViewNode.relY * focalAbsTransform.scale;
                    const angleRad = (focalAbsTransform.rotation * Math.PI) / 180; // Then rotate
                    const cosAngle = Math.cos(angleRad);
                    const sinAngle = Math.sin(angleRad);
                    const rotatedScaledRelX = scaledRelX * cosAngle - scaledRelY * sinAngle;
                    const rotatedScaledRelY = scaledRelX * sinAngle + scaledRelY * cosAngle;
                    const absX = focalAbsTransform.x + rotatedScaledRelX; // Then translate
                    const absY = focalAbsTransform.y + rotatedScaledRelY;
                    absoluteNode = {
                        id: storableViewNode.id, x: absX, y: absY,
                        width: storableViewNode.width, height: storableViewNode.height,
                        scale: absScale, rotation: absRotation
                    };
                }
                // DEBUG: Log conversion for getContext
                console.log(`[DEBUG getContext ${contextId}] Node ${nodeId} | RelIn: ${JSON.stringify(storableViewNode)} | Focal: ${JSON.stringify(focalAbsTransform)} | AbsOut: ${JSON.stringify(absoluteNode)}`);
                absoluteViewNodes.set(nodeId, absoluteNode);
            }

            // Convert StorableViewportSettings to ViewportSettings (absolute positions)
            let absoluteViewportSettings: ViewportSettings | undefined = undefined;
            if (storableContext.viewportSettings) {
                const relSettings = storableContext.viewportSettings;
                // Calculate absolute position correctly
                const scaledRelX = relSettings.relPosX * focalAbsTransform.scale; // Scale first
                const scaledRelY = relSettings.relPosY * focalAbsTransform.scale;
                const angleRad = (focalAbsTransform.rotation * Math.PI) / 180; // Then rotate
                const cosAngle = Math.cos(angleRad);
                const sinAngle = Math.sin(angleRad);
                const rotatedScaledRelX = scaledRelX * cosAngle - scaledRelY * sinAngle;
                const rotatedScaledRelY = scaledRelX * sinAngle + scaledRelY * cosAngle;
                const absPosX = focalAbsTransform.x + rotatedScaledRelX; // Then translate
                const absPosY = focalAbsTransform.y + rotatedScaledRelY;
                absoluteViewportSettings = {
                    scale: relSettings.scale, // Scale is absolute
                    posX: absPosX,
                    posY: absPosY
                };
            }

            return {
                id: storableContext.id,
                viewNodes: absoluteViewNodes,
                viewportSettings: absoluteViewportSettings // Load absolute viewport settings
            };
        }
        return undefined;
    }

    // getContexts needs similar conversion logic for viewportSettings
    async getContexts(focalAbsTransforms: Map<NodeId, AbsoluteTransform>): Promise<Context[]> {
        const db = await this.dbPromise;
        const storableContexts = await db.getAll('contexts') as StorableContext[];
        return storableContexts.map(storable => {
            const focalAbsTransform = focalAbsTransforms.get(storable.id) ?? ROOT_TRANSFORM;
            const absoluteViewNodes = new Map<NodeId, ViewNode>();
            // Convert ViewNodes
            for (const [nodeId, storableViewNode] of storable.viewNodes) {
                let absoluteNode: ViewNode;
                 if (nodeId === storable.id) {
                    absoluteNode = {
                        id: storableViewNode.id, x: focalAbsTransform.x, y: focalAbsTransform.y,
                        width: storableViewNode.width, height: storableViewNode.height,
                        scale: focalAbsTransform.scale, rotation: focalAbsTransform.rotation
                    };
                 } else {
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
                        id: storableViewNode.id, x: absX, y: absY,
                        width: storableViewNode.width, height: storableViewNode.height,
                        scale: absScale, rotation: absRotation
                    };
                 }
                absoluteViewNodes.set(nodeId, absoluteNode);
            }

            // Convert ViewportSettings
            let absoluteViewportSettings: ViewportSettings | undefined = undefined;
            if (storable.viewportSettings) {
                const relSettings = storable.viewportSettings;
                const scaledRelX = relSettings.relPosX * focalAbsTransform.scale;
                const scaledRelY = relSettings.relPosY * focalAbsTransform.scale;
                const angleRad = (focalAbsTransform.rotation * Math.PI) / 180;
                const cosAngle = Math.cos(angleRad);
                const sinAngle = Math.sin(angleRad);
                const rotatedScaledRelX = scaledRelX * cosAngle - scaledRelY * sinAngle;
                const rotatedScaledRelY = scaledRelX * sinAngle + scaledRelY * cosAngle;
                const absPosX = focalAbsTransform.x + rotatedScaledRelX;
                const absPosY = focalAbsTransform.y + rotatedScaledRelY;
                absoluteViewportSettings = {
                    scale: relSettings.scale,
                    posX: absPosX,
                    posY: absPosY
                };
            }

            return {
                id: storable.id,
                viewNodes: absoluteViewNodes,
                viewportSettings: absoluteViewportSettings
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
    contexts: {
        key: NodeId;
        value: StorableContext; // Stores relative StorableViewNodes and StorableViewportSettings
    };
}

let localAdapterInstance: LocalAdapter | null = null;
if (typeof window !== 'undefined') {
    localAdapterInstance = new LocalAdapter();
}
export const localAdapter = localAdapterInstance;
