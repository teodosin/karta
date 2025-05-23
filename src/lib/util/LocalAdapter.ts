import * as idb from 'idb';
import type { 
	DataNode,
	KartaEdge,
	Context,
	NodeId,
	StorableContext,
	StorableViewNode,
	StorableViewportSettings,
	AssetData,
	KartaExportData
} from '../types/types';
import type { PersistenceService } from './PersistenceService';


// Define default transform for root context, not needed anymore?
// const ROOT_TRANSFORM: AbsoluteTransform = { x: 0, y: 0, scale: 1 };

class LocalAdapter implements PersistenceService {
	private dbPromise: Promise<idb.IDBPDatabase<KartaDB>>;
	private objectUrlMap = new Map<string, string>(); // Tracks generated Object URLs { nodeId: objectUrl }

	constructor() {
		const startTime = performance.now();

		// DB Version 1: Initial schema with all stores and indexes
		this.dbPromise = idb.openDB<KartaDB>('karta-db', 1, { // Set version to 1
			upgrade(db, oldVersion, newVersion, tx, event) {
				// Create 'nodes' store with indexes if it doesn't exist
				if (!db.objectStoreNames.contains('nodes')) {
					const nodeStore = db.createObjectStore('nodes', { keyPath: 'id' });
					nodeStore.createIndex('path_idx', 'path', { unique: true });
					// Removed isSearchable_idx creation
				}

				// Create 'edges' store with indexes if it doesn't exist
				if (!db.objectStoreNames.contains('edges')) {
					const edgeStore = db.createObjectStore('edges', { keyPath: 'id' });
					edgeStore.createIndex('source_idx', 'source', { unique: false });
					edgeStore.createIndex('target_idx', 'target', { unique: false });
				}

				// Create 'contexts' store if it doesn't exist
				if (!db.objectStoreNames.contains('contexts')) {
					db.createObjectStore('contexts', { keyPath: 'id' });
				}

				// Create 'assets' store if it doesn't exist
				if (!db.objectStoreNames.contains('assets')) {
					db.createObjectStore('assets'); // Key is assetId (usually nodeId)
				}
				// Removed all oldVersion checks
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
	 * Retrieves all unique node paths for searchable nodes.
	 * Uses the isSearchable index and filters based on the isSearchable flag.
	 * @returns A promise that resolves with an array of searchable node paths.
	 */
	async getAllNodePaths(): Promise<string[]> {
		try {
			const db = await this.dbPromise;
			const tx = db.transaction('nodes', 'readonly');
			const index = tx.objectStore('nodes').index('path_idx'); // Use the path index
			const paths: string[] = [];
			// Open a key cursor on the index to iterate only through keys (paths)
			let cursor = await index.openKeyCursor();
			while (cursor) {
				// cursor.key is the path string
				if (typeof cursor.key === 'string') { // Ensure the key is a string
					paths.push(cursor.key);
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

	async deleteContext(contextId: NodeId): Promise<void> {
		try {
			const db = await this.dbPromise;
			const tx = db.transaction('contexts', 'readwrite');
			await tx.objectStore('contexts').delete(contextId);
			await tx.done;
		} catch (error) {
			console.error(`[LocalAdapter] Error deleting context ${contextId}:`, error);
			throw error; // Re-throw error to be handled by caller
		}
	}

	async getAllContextPaths(): Promise<Map<NodeId, string>> {
		console.log("[LocalAdapter::getAllContextPaths] Starting..."); // ADDED LOG
		try {
			const contextIds = await this.getAllContextIds();
			console.log(`[LocalAdapter::getAllContextPaths] Found ${contextIds.length} context IDs.`); // ADDED LOG
			if (contextIds.length === 0) {
				console.log("[LocalAdapter::getAllContextPaths] No context IDs found, returning empty map."); // ADDED LOG
				return new Map();
			}
			const nodesMap = await this.getDataNodesByIds(contextIds);
			console.log(`[LocalAdapter::getAllContextPaths] Fetched ${nodesMap.size} nodes corresponding to context IDs.`); // ADDED LOG
			const pathsMap = new Map<NodeId, string>();
			for (const [nodeId, nodeData] of nodesMap.entries()) {
				if (nodeData?.path) { // Ensure nodeData and path exist
					pathsMap.set(nodeId, nodeData.path);
				} else {
					// ENHANCED LOG
					console.warn(`[LocalAdapter::getAllContextPaths] Node data or path missing for context ID ${nodeId}. Node data found: ${!!nodeData}`);
				}
			}
			console.log(`[LocalAdapter::getAllContextPaths] Successfully built paths map with ${pathsMap.size} entries.`); // ADDED LOG
			return pathsMap;
		} catch (error) {
			// MODIFIED LOG to include full error object
			console.error("[LocalAdapter::getAllContextPaths] Error getting all context paths:", error);
			return new Map(); // Return empty map on error
		}
	}

	// --- Export/Import Methods ---

	/**
	 * Helper function to convert Blob to Data URL.
	 * @param blob The Blob to convert.
	 * @returns A promise that resolves with the Data URL string.
	 */
	private async blobToDataURL(blob: Blob): Promise<string> {
		return new Promise((resolve, reject) => {
			const reader = new FileReader();
			reader.onload = () => resolve(reader.result as string);
			reader.onerror = (error) => reject(error);
			reader.readAsDataURL(blob);
		});
	}

	/**
	 * Retrieves all data required for export.
	 * Fetches nodes, edges, contexts, and assets, converting assets to Data URLs.
	 * @returns A promise that resolves with the structured export data.
	 */
	async getExportData(): Promise<KartaExportData> {
		try {
			const db = await this.dbPromise;
			const tx = db.transaction(['nodes', 'edges', 'contexts', 'assets'], 'readonly');

			const nodes = await tx.objectStore('nodes').getAll();
			const edges = await tx.objectStore('edges').getAll();
			const contexts = await tx.objectStore('contexts').getAll();
			const assetKeys = await tx.objectStore('assets').getAllKeys();
			const assetDataPromises = assetKeys.map(key => tx.objectStore('assets').get(key));
			const assetDataValues = await Promise.all(assetDataPromises);

			await tx.done; // Complete the read transaction

			// Convert assets to the export format with Data URLs
			const exportAssets = await Promise.all(
				assetKeys.map(async (key, index) => {
					const asset = assetDataValues[index];
					if (!asset || !asset.blob) {
						console.warn(`[LocalAdapter] Asset data or blob missing for key ${key} during export.`);
						return null; // Skip assets without blobs
					}
					const dataUrl = await this.blobToDataURL(asset.blob);
					return {
						assetId: key as string,
						mimeType: asset.mimeType,
						name: asset.name,
						dataUrl: dataUrl
					};
				})
			);

			// Filter out any null assets (where blob was missing)
			const filteredAssets = exportAssets.filter(asset => asset !== null) as KartaExportData['assets'];

			const exportData: KartaExportData = {
				version: 1,
				exportedAt: new Date().toISOString(),
				nodes: nodes,
				edges: edges,
				contexts: contexts,
				assets: filteredAssets
			};

			return exportData;

		} catch (error) {
			console.error("[LocalAdapter] Error getting export data:", error);
			throw error; // Re-throw the error to be handled by the caller
		}
	}
	/**
		 * Helper function to convert Data URL string back to Blob.
		 * @param dataUrl The Data URL string.
		 * @returns A Blob object or null if conversion fails.
		 */
	private dataURLtoBlob(dataUrl: string): Blob | null {
		try {
			const arr = dataUrl.split(',');
			if (arr.length < 2) return null;
			const mimeMatch = arr[0].match(/:(.*?);/);
			if (!mimeMatch || mimeMatch.length < 2) return null;
			const mime = mimeMatch[1];
			const bstr = atob(arr[1]);
			let n = bstr.length;
			const u8arr = new Uint8Array(n);
			while (n--) {
				u8arr[n] = bstr.charCodeAt(n);
			}
			return new Blob([u8arr], { type: mime });
		} catch (error) {
			console.error("[LocalAdapter] Error converting Data URL to Blob:", error);
			return null;
		}
	}

	/**
	 * Imports data from a KartaExportData object.
	 * Clears existing database content and replaces it with the imported data.
	 * @param data The KartaExportData object to import.
	 * @returns A promise that resolves when the import is complete.
	 */
	async importData(data: KartaExportData): Promise<void> {
		// Basic validation
		if (!data || data.version !== 1 || !Array.isArray(data.nodes) || !Array.isArray(data.edges) || !Array.isArray(data.contexts) || !Array.isArray(data.assets)) {
			throw new Error("Invalid import data format or version.");
		}

		console.log("[LocalAdapter] Starting data import...");
		const db = await this.dbPromise;
		const tx = db.transaction(['nodes', 'edges', 'contexts', 'assets'], 'readwrite');

		try {
			const nodeStore = tx.objectStore('nodes');
			const edgeStore = tx.objectStore('edges');
			const contextStore = tx.objectStore('contexts');
			const assetStore = tx.objectStore('assets');

			// Clear existing data
			console.log("[LocalAdapter] Clearing existing data...");
			await Promise.all([
				nodeStore.clear(),
				edgeStore.clear(),
				contextStore.clear(),
				assetStore.clear()
			]);
			console.log("[LocalAdapter] Existing data cleared.");

			// Import assets
			console.log(`[LocalAdapter] Importing ${data.assets.length} assets...`);
			const assetImportPromises = data.assets.map(asset => {
				const blob = this.dataURLtoBlob(asset.dataUrl);
				if (blob) {
					const assetData: AssetData = {
						blob: blob,
						mimeType: asset.mimeType,
						name: asset.name
					};
					return assetStore.put(assetData, asset.assetId);
				} else {
					console.warn(`[LocalAdapter] Failed to convert Data URL to Blob for asset ${asset.assetId}. Skipping.`);
					return Promise.resolve(); // Skip problematic assets
				}
			});
			await Promise.all(assetImportPromises);
			console.log("[LocalAdapter] Assets imported.");

			// Import nodes
			console.log(`[LocalAdapter] Importing ${data.nodes.length} nodes...`);
			const nodeImportPromises = data.nodes.map(node => nodeStore.put(node));
			await Promise.all(nodeImportPromises);
			console.log("[LocalAdapter] Nodes imported.");

			// Import edges
			console.log(`[LocalAdapter] Importing ${data.edges.length} edges...`);
			const edgeImportPromises = data.edges.map(edge => edgeStore.put(edge));
			await Promise.all(edgeImportPromises);
			console.log("[LocalAdapter] Edges imported.");

			// Import contexts
			console.log(`[LocalAdapter] Importing ${data.contexts.length} contexts...`);
			const contextImportPromises = data.contexts.map(context => contextStore.put(context));
			await Promise.all(contextImportPromises);
			console.log("[LocalAdapter] Contexts imported.");

			await tx.done;
			console.log("[LocalAdapter] Data import completed successfully.");

			// Clear Object URL cache after import as old URLs are invalid
			this.cleanupObjectUrls();

		} catch (error) {
			console.error("[LocalAdapter] Error during data import:", error);
			// Attempt to abort the transaction on error
			if (tx.abort) {
				tx.abort();
			}
			throw error; // Re-throw the error
		}
	}

	// getContexts needs similar conversion logic for viewportSettings
	// Removed unused getContexts function
}

// Define database schema (for TypeScript type checking)
// TODO: Update PersistenceService interface definition to include deleteContext and getAllContextPaths
interface KartaDB extends idb.DBSchema {
	nodes: {
		key: string;
		value: DataNode;
		indexes: { 'path_idx': string; 'isSearchable_idx': any }; // Changed boolean to any for isSearchable index type
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
