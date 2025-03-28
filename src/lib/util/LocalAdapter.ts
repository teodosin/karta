import * as idb from 'idb';
import type { KartaNode, KartaEdge } from '../types/types'; // Import both types from types.ts

interface PersistenceService {
    saveNode(node: KartaNode): Promise<void>;
    getNode(nodeId: string): Promise<KartaNode | undefined>;
    deleteNode(nodeId: string): Promise<void>;

    saveEdge(edge: KartaEdge): Promise<void>;
    getEdge(edgeId: string): Promise<KartaEdge | undefined>;
    getEdges(): Promise<KartaEdge[]>;
    deleteEdge(edgeId: string): Promise<void>;
    // ... similar methods for layouts will be added later
}

class LocalAdapter implements PersistenceService {
    private dbPromise: Promise<idb.IDBPDatabase<KartaDB>>;

    constructor() {
        this.dbPromise = idb.openDB<KartaDB>('karta-db', 1, {
            upgrade(db) {
                db.createObjectStore('nodes', { keyPath: 'id' });
                db.createObjectStore('edges', { keyPath: 'id' });
                // db.createObjectStore('layouts', { keyPath: 'nodeId' }); // Layouts will be added later
            },
        });
    }

    async saveNode(node: KartaNode): Promise<void> {
        const db = await this.dbPromise;
        const tx = db.transaction('nodes', 'readwrite');
        const store = tx.objectStore('nodes');
        await store.put(node);
        await tx.done;
    }

    async getNode(nodeId: string): Promise<KartaNode | undefined> {
        const db = await this.dbPromise;
        return db.get('nodes', nodeId) as Promise<KartaNode | undefined>;
    }

    async deleteNode(nodeId: string): Promise<void> {
        const db = await this.dbPromise;
        const tx = db.transaction('nodes', 'readwrite');
        const store = tx.objectStore('nodes');
        await store.delete(nodeId);
        await tx.done;
    }

    async getNodes(): Promise<KartaNode[]> {
        const db = await this.dbPromise;
        return db.getAll('nodes') as Promise<KartaNode[]>;
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

} // Closing brace for LocalAdapter class was missing

// Define database schema (for TypeScript type checking)
interface KartaDB extends idb.DBSchema {
    nodes: {
        key: string;
        value: KartaNode;
    };
    edges: {
        key: string;
        value: KartaEdge; // Use KartaEdge type here
    };
    // layouts: {
    //     key: string;
    //     value: any;
    // };
}


let localAdapterInstance: LocalAdapter | null = null;

if (typeof window !== 'undefined') {
    localAdapterInstance = new LocalAdapter();
}

export const localAdapter = localAdapterInstance;
