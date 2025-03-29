import * as idb from 'idb';
// Import Context, ViewNode, NodeId as well
import type { DataNode, KartaEdge, Context, ViewNode, NodeId } from '../types/types';

// Interface for the storable version of Context (Map converted to Array)
interface StorableContext {
    id: NodeId;
    viewNodes: [NodeId, ViewNode][]; // Store as array of [key, value] tuples
}

interface PersistenceService {
    saveNode(node: DataNode): Promise<void>;
    getNode(nodeId: string): Promise<DataNode | undefined>; // Use DataNode
    deleteNode(nodeId: string): Promise<void>;
    getNodes(): Promise<DataNode[]>; // Add getNodes signature using DataNode

    saveEdge(edge: KartaEdge): Promise<void>;
    getEdge(edgeId: string): Promise<KartaEdge | undefined>;
    getEdges(): Promise<KartaEdge[]>;
    deleteEdge(edgeId: string): Promise<void>;

    // Add Context methods
    saveContext(context: Context): Promise<void>;
    getContext(contextId: NodeId): Promise<Context | undefined>;
    getContexts(): Promise<Context[]>;
}

class LocalAdapter implements PersistenceService {
    private dbPromise: Promise<idb.IDBPDatabase<KartaDB>>;

    constructor() {
        this.dbPromise = idb.openDB<KartaDB>('karta-db', 1, {
            upgrade(db, oldVersion) { // Add oldVersion parameter
                if (oldVersion < 1) { // Check oldVersion before creating stores
                    if (!db.objectStoreNames.contains('nodes')) {
                        db.createObjectStore('nodes', { keyPath: 'id' });
                    }
                    if (!db.objectStoreNames.contains('edges')) {
                        db.createObjectStore('edges', { keyPath: 'id' });
                    }
                }
                 // Add contexts store in version 2 (or adjust version number as needed)
                 // Assuming current version is 1, upgrade to 2
                 if (oldVersion < 2) {
                    if (!db.objectStoreNames.contains('contexts')) {
                         db.createObjectStore('contexts', { keyPath: 'id' });
                    }
                 }
            },
        });
        // Update DB version number
        this.dbPromise = idb.openDB<KartaDB>('karta-db', 2, { // Increment version to 2
            upgrade(db, oldVersion) {
                if (oldVersion < 1) {
                    if (!db.objectStoreNames.contains('nodes')) {
                        db.createObjectStore('nodes', { keyPath: 'id' });
                    }
                    if (!db.objectStoreNames.contains('edges')) {
                        db.createObjectStore('edges', { keyPath: 'id' });
                    }
                }
                 if (oldVersion < 2) {
                    if (!db.objectStoreNames.contains('contexts')) {
                         db.createObjectStore('contexts', { keyPath: 'id' });
                    }
                 }
            },
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

    async saveContext(context: Context): Promise<void> {
        const db = await this.dbPromise;
        const tx = db.transaction('contexts', 'readwrite');
        const store = tx.objectStore('contexts');
        // Convert Map to Array for storage
        const storableContext: StorableContext = {
            ...context,
            viewNodes: Array.from(context.viewNodes.entries()),
        };
        await store.put(storableContext);
        await tx.done;
    }

    async getContext(contextId: NodeId): Promise<Context | undefined> {
        const db = await this.dbPromise;
        const storableContext = await db.get('contexts', contextId) as StorableContext | undefined;
        if (storableContext) {
            // Convert Array back to Map
            return {
                ...storableContext,
                viewNodes: new Map(storableContext.viewNodes),
            };
        }
        return undefined;
    }

    async getContexts(): Promise<Context[]> {
        const db = await this.dbPromise;
        const storableContexts = await db.getAll('contexts') as StorableContext[];
        // Convert Array back to Map for each context
        return storableContexts.map(storable => ({
            ...storable,
            viewNodes: new Map(storable.viewNodes),
        }));
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
        value: StorableContext; // Store the array version
    };
}


let localAdapterInstance: LocalAdapter | null = null;

if (typeof window !== 'undefined') {
    localAdapterInstance = new LocalAdapter();
}

export const localAdapter = localAdapterInstance;
