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
	getAllContextIds(): Promise<NodeId[]>; // Added this method

	// Asset methods
	saveAsset(assetId: string, assetData: AssetData): Promise<void>; // Added this method
	getAsset(assetId: string): Promise<AssetData | undefined>; // Added this method
	deleteAsset(assetId: string): Promise<void>; // Added this method
	getAssetObjectUrl(assetId: string): Promise<string | null>; // Added this method
}

class LocalAdapter implements PersistenceService {
	private dbPromise: Promise<idb.IDBPDatabase<KartaDB>>;
	private objectUrlMap = new Map<string, string>(); // Tracks generated Object URLs { nodeId: objectUrl }

	constructor() {
		const startTime = performance.now();

		// DB Version 6: Added path_idx index to nodes store
		this.dbPromise = idb.openDB<KartaDB>('karta-db', 6, {
			upgrade(db, oldVersion, newVersion, tx, event) {
				// Ensure 'nodes' store exists and has its indexes
				if (!db.objectStoreNames.contains('nodes')) {
					const nodeStore = db.createObjectStore('nodes', { keyPath: 'id' });
					// Add path index if it doesn't exist
					if (!nodeStore.indexNames.contains('path_idx')) {
						nodeStore.createIndex('path_idx', 'path', { unique: true }); // Path should be unique
					}
				}

				// Ensure 'edges' store exists and has its indexes
				if (!db.objectStoreNames.contains('edges')) {
					const edgeStore = db.createObjectStore('edges', { keyPath: 'id' });
					// Create indexes immediately after store creation if they don't exist
					if (!edgeStore.indexNames.contains('source_idx')) {
						edgeStore.createIndex('source_idx', 'source', { unique: false });
					}
					if (!edgeStore.indexNames.contains('target_idx')) {
						edgeStore.createIndex('target_idx', 'target', { unique: false });
					}
				}

				// Ensure 'contexts' store exists
				if (!db.objectStoreNames.contains('contexts')) {
					db.createObjectStore('contexts', { keyPath: 'id' });
				}
				// Add path index if upgrading from v1, v2, v3, v4, or v5 (where nodes store exists but path_idx might not)
				if (oldVersion >= 1 && oldVersion < 6) {
					if (db.objectStoreNames.contains('nodes')) {
						const nodeStore = tx.objectStore('nodes');
						if (!nodeStore.indexNames.contains('path_idx')) {
							nodeStore.createIndex('path_idx', 'path', { unique: true });
						}
					} else { console.warn('[LocalAdapter] Cannot add path_idx: "nodes" store does not exist.'); }
				}
				// Add edge indexes if upgrading from v1, v2, or v3
				if (oldVersion >= 1 && oldVersion < 4) {
					if (db.objectStoreNames.contains('edges')) {
						const edgeStore = tx.objectStore('edges');
						if (!edgeStore.indexNames.contains('source_idx')) {
							edgeStore.createIndex('source_idx', 'source', { unique: false });
						}
						if (!edgeStore.indexNames.contains('target_idx')) {
							edgeStore.createIndex('target_idx', 'target', { unique: false });
						}
					} else { console.warn('[LocalAdapter] Cannot add edge indexes: "edges" store does not exist.'); }
				}
				// Ensure 'assets' store exists
				if (!db.objectStoreNames.contains('assets')) {
					db.createObjectStore('assets'); // Key is assetId (usually nodeId)
				}
			},
			blocked() { console.error('[LocalAdapter] IDB blocked. Close other tabs accessing the DB.'); },
			blocking() { console.warn('[LocalAdapter] IDB blocking. Connection will close.'); },
			terminated() { console.warn('[LocalAdapter] IDB connection terminated unexpectedly.'); }
		});

		this.dbPromise.then(db => {
			const endTime = performance.now();
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
		// Return node data directly without generating Object URL here
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
		// Return nodes directly without generating Object URLs here
		return db.getAll('nodes');
	}

	async getDataNodesByIds(nodeIds: NodeId[]): Promise<Map<NodeId, DataNode>> {
		const db = await this.dbPromise;
		const tx = db.transaction('nodes', 'readonly');
		const store = tx.objectStore('nodes');
		const nodesMap = new Map<NodeId, DataNode>();
		await Promise.all(nodeIds.map(async (id) => {
			const node = await store.get(id);
			// Get node data directly without generating Object URL here
			if (node) {
				nodesMap.set(id, node);
			}
		}));
		await tx.done;
		return nodesMap;
	}

	/**
	 * Checks if a node with the given name exists in the database.
	 * Note: This iterates through all nodes, as there is no index on name.
	 * @param name The name to check.
	 * @returns A promise that resolves with true if the name exists, false otherwise.
	 */
	async checkNameExists(name: string): Promise<boolean> {
		try {
			const db = await this.dbPromise;
			const allNodes = await db.getAll('nodes');
			const exists = allNodes.some(node => node.attributes?.name === name);
			return exists;
		} catch (error) {
			console.error(`[LocalAdapter] Error checking if name "${name}" exists:`, error);
			return false; // Assume it doesn't exist on error to avoid blocking creation? Or throw? Let's return false.
		}
	}

	/**
	 * Retrieves all unique node paths using the path_idx index keys.
	 * @returns A promise that resolves with an array of node paths.
	 */
	async getAllNodePaths(): Promise<string[]> {
		try {
			const db = await this.dbPromise;
			const tx = db.transaction('nodes', 'readonly');
			const index = tx.objectStore('nodes').index('path_idx');
			const paths: string[] = [];
			let cursor = await index.openKeyCursor();
			while (cursor) {
				// cursor.key will be the path string
				if (cursor.key != null) { // Ensure path is not null/undefined
					paths.push(cursor.key as string);
				}
				cursor = await cursor.continue();
			}
			await tx.done;
			return paths;
		} catch (error) {
			console.error("[LocalAdapter] Error getting all node paths:", error);
			return []; // Return empty array on error
		}
	}

	/**
	 * Retrieves a single DataNode by its path using the path_idx index.
	 * @param path The path of the node to retrieve.
	 * @returns A promise that resolves with the DataNode or undefined if not found.
	 */
	async getDataNodeByPath(path: string): Promise<DataNode | undefined> {
		try {
			const db = await this.dbPromise;
			const tx = db.transaction('nodes', 'readonly');
			const index = tx.objectStore('nodes').index('path_idx');
			const node = await index.get(path);
			await tx.done;
			return node;
		} catch (error) {
			console.error(`[LocalAdapter] Error getting node by path "${path}":`, error);
			return undefined; // Return undefined on error
		}
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
		}
	}

	/**
	 * Retrieves or generates an Object URL for a given asset ID.
	 * Checks the cache first, then fetches from DB if necessary.
	 * Returns null if the asset or blob is not found.
	 */
	async getAssetObjectUrl(assetId: string): Promise<string | null> {
		// 1. Check cache
		const cachedUrl = this.objectUrlMap.get(assetId);
		if (cachedUrl) {
			// console.log(`[LocalAdapter] Returning cached Object URL for asset ${assetId}`);
			return cachedUrl;
		}

		// 2. Fetch asset data from DB
		// console.log(`[LocalAdapter] Object URL for asset ${assetId} not cached. Fetching from DB...`);
		const assetData = await this.getAsset(assetId);

		if (assetData?.blob) {
			// 3. Generate new Object URL
			try {
				const newObjectUrl = URL.createObjectURL(assetData.blob);
				this.objectUrlMap.set(assetId, newObjectUrl); // Cache the new URL
				// console.log(`[LocalAdapter] Generated and cached new Object URL for asset ${assetId}`);
				return newObjectUrl;
			} catch (error) {
				console.error(`[LocalAdapter] Error creating Object URL for asset ${assetId}:`, error);
				return null; // Return null on generation error
			}
		} else {
			console.warn(`[LocalAdapter] Asset data or blob not found for assetId ${assetId} when requesting Object URL.`);
			return null; // Return null if asset or blob doesn't exist
		}
	}

	// Method to clean up all generated Object URLs
	// Use arrow function syntax to ensure 'this' context is correct when used as event listener
	private cleanupObjectUrls = (): void => {
		this.objectUrlMap.forEach((url, nodeId) => {
			URL.revokeObjectURL(url);
			// console.log(`[LocalAdapter] Revoked Object URL for node ${nodeId}`);
		});
		this.objectUrlMap.clear();
	};

	// --- Context Methods ---

	async saveContext(context: Context): Promise<void> {
		const db = await this.dbPromise;
		const tx = db.transaction('contexts', 'readwrite');
		const store = tx.objectStore('contexts');
		const focalNode = context.viewNodes.get(context.id);
		// Access focal node state via the tween
		const focalNodeState = focalNode?.state.current;
		if (!focalNodeState) throw new Error(`Focal node ${context.id} state not found in context being saved`);

		// Convert ViewNodes (containing Tweens) to StorableViewNodes (relative positions)
		const storableViewNodes: [NodeId, StorableViewNode][] = [];
		for (const [nodeId, viewNode] of context.viewNodes.entries()) {
			const nodeState = viewNode.state.current; // Get current state from tween
			let storableNode: StorableViewNode;
			if (nodeId === context.id) {
				// Focal node is always relative origin
				storableNode = {
					id: viewNode.id,
					relX: 0,
					relY: 0,
					width: nodeState.width,
					height: nodeState.height,
					relScale: 1,
					rotation: nodeState.rotation,
					attributes: viewNode.attributes // Include generic attributes
				};
			} else {
				// Calculate relative properties based on focal node's current state
				const relScale = nodeState.scale / focalNodeState.scale;
				const dx = nodeState.x - focalNodeState.x;
				const dy = nodeState.y - focalNodeState.y;
				// Simplified relative position calculation (no rotation considered for offset)
				const relX = dx / focalNodeState.scale;
				const relY = dy / focalNodeState.scale;
				storableNode = {
					id: viewNode.id,
					relX,
					relY,
					width: nodeState.width,
					height: nodeState.height,
					relScale,
					rotation: nodeState.rotation,
					attributes: viewNode.attributes // Include generic attributes
				};
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
		indexes: { 'path_idx': string }; // Define only the path index
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
