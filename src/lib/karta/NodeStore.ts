// NodeStore: Manages DataNode state and related actions.

import { writable, get } from 'svelte/store';
import type { NodeId, DataNode, AssetData } from '../types/types';
import { v4 as uuidv4 } from 'uuid';
import { localAdapter } from '../util/LocalAdapter'; // Assuming LocalAdapter is initialized elsewhere
import { getDefaultAttributesForType } from '$lib/node_types/registry';
// Import necessary stores when they are fully defined
// import { contexts, addViewNodeToCurrentContext } from './ContextStore'; // Placeholder
// import { ROOT_NODE_ID } from './ContextStore'; // Placeholder

// Define ROOT_NODE_ID locally for now, move later if needed
const ROOT_NODE_ID = '00000000-0000-0000-0000-000000000000';

export const nodes = writable<Map<NodeId, DataNode>>(new Map());

// --- Internal Helpers ---

/** Generalized function to ensure a DataNode exists */
export async function _ensureDataNodeExists(nodeId: NodeId): Promise<DataNode | null> {
	// Check in-memory store first
	const existingNode = get(nodes).get(nodeId);
	if (existingNode) {
		return existingNode;
	}

	// If not in memory, check persistence
	if (!localAdapter) {
		console.error(`[_ensureDataNodeExists] LocalAdapter not available while checking for ${nodeId}`);
		return null;
	}
	try {
		let dataNode = await localAdapter.getNode(nodeId);
		if (nodeId === ROOT_NODE_ID) {
			console.log(`[_ensureDataNodeExists] Root node check: Found in DB?`, !!dataNode, dataNode ? `Existing ntype: ${dataNode.ntype}` : '');
		}
		if (!dataNode) {
			console.warn(`[_ensureDataNodeExists] DataNode ${nodeId} not found. Creating default.`);
			const now = Date.now();
			// Determine default properties based on whether it's the root node
			const isRoot = nodeId === ROOT_NODE_ID;
			const defaultName = isRoot ? 'root' : `node-${nodeId.substring(0, 8)}`;
			const defaultPath = isRoot ? '/root' : `/${defaultName}`;
			const defaultNtype = isRoot ? 'root' : 'generic';
			if (isRoot) {
				console.log(`[_ensureDataNodeExists] Root node creation: Assigning ntype: ${defaultNtype}`);
			}

			dataNode = {
				id: nodeId,
				ntype: defaultNtype, // Set type correctly
				createdAt: now,
				modifiedAt: now,
				path: defaultPath, // Set path correctly
				attributes: { name: defaultName, ...(isRoot && { isSystemNode: true }) }, // Set name and system flag for root
			};
			await localAdapter.saveNode(dataNode);
			console.log(`[_ensureDataNodeExists] Default DataNode ${nodeId} created and saved.`);
		}
		// Add to in-memory store after ensuring existence
		nodes.update(n => n.set(nodeId, dataNode!));
		return dataNode;
	} catch (error) {
		console.error(`[_ensureDataNodeExists] Error ensuring DataNode ${nodeId} exists:`, error);
		return null;
	}
}


// --- Public Actions ---

// Node Creation
export async function createNodeAtPosition(
	canvasX: number,
	canvasY: number,
	ntype: string = 'text',
	attributes: Record<string, any> = {},
	initialWidth?: number, // Optional initial width
	initialHeight?: number // Optional initial height
): Promise<NodeId | null> {
	const newNodeId: NodeId = uuidv4();
	const now = Date.now();
	let baseName = attributes.name || ntype;
	let finalName = baseName;
	let counter = 2;

	// Ensure name uniqueness
	if (localAdapter) {
		while (await localAdapter.checkNameExists(finalName)) {
			finalName = `${baseName}${counter}`; counter++;
		}
	} else { console.warn("[NodeStore] LocalAdapter not ready, cannot check for duplicate names."); }

	// 1. Create DataNode
	const newNodeData: DataNode = {
		id: newNodeId, ntype: ntype, createdAt: now, modifiedAt: now,
		path: `/${finalName}`, // Simple path for now
		attributes: { ...attributes, name: finalName },
	};

	// 2. Update nodes store
	nodes.update(n => n.set(newNodeId, newNodeData));

	// 3. Persist DataNode
	let savedNode = false;
	if (localAdapter) {
		try {
			await localAdapter.saveNode(newNodeData);
			savedNode = true;
			console.log(`[NodeStore] DataNode ${newNodeId} saved.`);
		} catch (error) {
			console.error("[NodeStore] Error saving new DataNode:", error);
			// Remove from store if save failed?
			nodes.update(n => { n.delete(newNodeId); return n; });
			return null;
		}
	} else {
		console.warn("[NodeStore] LocalAdapter not initialized, persistence disabled for new node.");
		// Keep in store for non-persistent use
	}

	// 4. Add ViewNode to Context (Requires ContextStore)
	try {
		// TODO: Replace with import from ContextStore when available
		console.log(`[NodeStore] Placeholder: Would call ContextStore.addViewNodeToCurrentContext(${newNodeId}, ...)`);
		// await addViewNodeToCurrentContext(newNodeId, canvasX, canvasY, initialWidth, initialHeight);
		return newNodeId; // Return ID if ViewNode addition is assumed successful (or handled elsewhere)
	} catch (error) {
		console.error("[NodeStore] Error adding ViewNode to context:", error);
		// If adding ViewNode fails, should we delete the DataNode?
		if (savedNode && localAdapter) {
			console.warn("[NodeStore] Rolling back DataNode creation due to ViewNode error.");
			await localAdapter.deleteNode(newNodeId);
			nodes.update(n => { n.delete(newNodeId); return n; });
		}
		return null;
	}
}

/**
 * Creates an ImageNode, saves its Blob asset, and sets the Object URL as src.
 * Assumes the base node structure is created first via createNodeAtPosition.
 */
export async function saveImageAssetAndUpdateNode(
	nodeId: NodeId,
	imageBlob: Blob,
	objectUrl: string, // Pre-generated Object URL
	assetName: string
): Promise<boolean> {
	if (!localAdapter) {
		console.error("[NodeStore - saveImageAssetAndUpdateNode] LocalAdapter not available.");
		URL.revokeObjectURL(objectUrl); // Revoke if we can't save
		return false;
	}

	try {
		// 1. Prepare asset data and save it using the nodeId as assetId
		const assetData: AssetData = { blob: imageBlob, mimeType: imageBlob.type, name: assetName };
		await localAdapter.saveAsset(nodeId, assetData);

		// 2. Update the node attributes with the Object URL and assetId
		await updateNodeAttributes(nodeId, { src: objectUrl, assetId: nodeId });

		console.log(`[NodeStore] Saved asset ${assetName} for ImageNode ${nodeId}`);
		return true;

	} catch (error) {
		console.error(`[NodeStore - saveImageAssetAndUpdateNode] Error saving asset or updating node ${nodeId}:`, error);
		// Don't delete the node here, as it might have existed before.
		// The caller (e.g., createImageNodeWithAsset) should handle node cleanup if needed.
		// Revoke the initially passed objectUrl as it might not have been tracked/revoked.
		URL.revokeObjectURL(objectUrl);
		console.log(`[NodeStore - saveImageAssetAndUpdateNode] Revoked initial objectUrl: ${objectUrl}`);
		return false;
	}
}


/**
 * Creates an ImageNode at the specified position using a Data URL.
 * This might be deprecated in favor of Blob storage.
 */
export async function createImageNodeFromDataUrl(position: { x: number, y: number }, dataUrl: string, width?: number, height?: number) {
	try {
		const newNodeId = await createNodeAtPosition(position.x, position.y, 'image', {}, width, height);
		if (!newNodeId) {
			console.error("[NodeStore] Failed to create base node for image paste.");
			return;
		}
		// Update the attributes with the image source
		await updateNodeAttributes(newNodeId, { src: dataUrl });
		console.log(`[NodeStore] Created ImageNode ${newNodeId} from Data URL at (${position.x}, ${position.y})`);
	} catch (error) {
		console.error("[NodeStore] Error creating image node from Data URL:", error);
	}
}

/**
 * Creates a TextNode at the specified position with the given text content.
 */
export async function createTextNodeFromPaste(position: { x: number, y: number }, text: string) {
	try {
		const newNodeId = await createNodeAtPosition(position.x, position.y, 'text');
		if (!newNodeId) {
			console.error("[NodeStore] Failed to create base node for text paste.");
			return;
		}
		// Update the attributes with the pasted text
		await updateNodeAttributes(newNodeId, { text: text });
		console.log(`[NodeStore] Created TextNode ${newNodeId} from paste at (${position.x}, ${position.y})`);
	} catch (error) {
		console.error("[NodeStore] Error creating text node from paste:", error);
	}
}

/**
 * Creates an ImageNode, saves its Blob asset, and sets the Object URL as src.
 * Coordinates node creation and asset saving.
 */
export async function createImageNodeWithAsset(
	position: { x: number, y: number },
	imageBlob: Blob,
	objectUrl: string, // Pre-generated Object URL
	assetName: string,
	initialWidth?: number,
	initialHeight?: number
): Promise<NodeId | null> {
	let newNodeId: NodeId | null = null;
	try {
		// 1. Create the base 'image' node
		newNodeId = await createNodeAtPosition(
			position.x,
			position.y,
			'image',
			{ alt: assetName }, // Set alt attribute initially
			initialWidth,
			initialHeight
		);

		if (!newNodeId) {
			throw new Error("Failed to create base node structure.");
		}

		// 2. Save the asset and update the node
		const success = await saveImageAssetAndUpdateNode(newNodeId, imageBlob, objectUrl, assetName);

		if (!success) {
			// If saving asset/updating failed, attempt to clean up the created node
			throw new Error("Failed to save asset or update node after base node creation.");
		}

		console.log(`[NodeStore] Successfully created ImageNode ${newNodeId} with asset ${assetName}`);
		return newNodeId;

	} catch (error) {
		console.error("[NodeStore - createImageNodeWithAsset] Error:", error);
		// Cleanup logic: If the base node was created but subsequent steps failed
		if (newNodeId && localAdapter) {
			console.log(`[NodeStore - createImageNodeWithAsset] Cleaning up partially created node ${newNodeId}`);
			await localAdapter.deleteNode(newNodeId); // Adapter should handle asset deletion/URL revoke
			nodes.update(n => { n.delete(newNodeId!); return n; });
			// TODO: Need mechanism to remove ViewNode from ContextStore if it was added
			console.log(`[NodeStore] Placeholder: Would need to remove ViewNode ${newNodeId} from ContextStore`);
		} else if (!localAdapter) {
			// If no adapter, still revoke the URL passed in
			URL.revokeObjectURL(objectUrl);
		}
		return null; // Indicate failure
	}
}


// Node Attribute Update
export async function updateNodeAttributes(nodeId: NodeId, newAttributes: Record<string, any>) {
	const currentNodes = get(nodes);
	const dataNode = currentNodes.get(nodeId);

	if (!dataNode) {
		console.warn(`[NodeStore - updateNodeAttributes] DataNode ${nodeId} not found in store.`);
		return;
	}

	// Prevent modifying system nodes (check might be redundant if creation prevents it)
	if (dataNode.attributes?.isSystemNode && Object.keys(newAttributes).some(key => key !== 'isSystemNode')) {
		console.warn(`[NodeStore - updateNodeAttributes] Attempted to modify attributes of system node ${nodeId}. Operation cancelled.`);
		return;
	}

	const oldName = dataNode.attributes?.name;
	const newName = newAttributes?.name;
	let attributesToSave = { ...dataNode.attributes, ...newAttributes }; // Merge old and new

	// Check for name change and uniqueness
	if (newName && newName.trim() && newName !== oldName) {
		const finalNewName = newName.trim();
		if (localAdapter) {
			let nameToSet = finalNewName;
			let nameExists = await localAdapter.checkNameExists(nameToSet);

			if (nameExists) {
				console.warn(`[NodeStore - updateNodeAttributes] Name "${nameToSet}" already exists. Finding next available name...`);
				const baseName = nameToSet;
				let counter = 2;
				while (nameExists) {
					nameToSet = `${baseName}${counter}`;
					nameExists = await localAdapter.checkNameExists(nameToSet);
					counter++;
				}
				console.log(`[NodeStore - updateNodeAttributes] Unique name found: "${nameToSet}"`);
			}
			attributesToSave.name = nameToSet;
		} else {
			console.warn("[NodeStore - updateNodeAttributes] LocalAdapter not ready, cannot check for duplicate names.");
			// Cancel the rename if uniqueness cannot be verified
			console.error("[NodeStore - updateNodeAttributes] LocalAdapter not available. Cannot verify name uniqueness. Rename cancelled.");
			return;
		}
	} else if (newName !== undefined && !newName.trim()) {
		console.warn(`[NodeStore - updateNodeAttributes] Attempted to rename node ${nodeId} to an empty name. Operation cancelled.`);
		return; // Prevent empty names
	}

	// Create updated node data only if attributes actually changed
	if (JSON.stringify(attributesToSave) === JSON.stringify(dataNode.attributes)) {
		console.log(`[NodeStore - updateNodeAttributes] No effective attribute changes for node ${nodeId}.`);
		return;
	}

	const updatedNodeData: DataNode = {
		...dataNode,
		attributes: attributesToSave,
		modifiedAt: Date.now(),
		// path: `/${attributesToSave.name}` // Decide if path should sync with name
	};

	// Update the store
	nodes.update(n => n.set(nodeId, updatedNodeData));
	console.log(`[NodeStore - updateNodeAttributes] Updated attributes for node ${nodeId}:`, attributesToSave);

	// Persist changes
	if (localAdapter) {
		try {
			await localAdapter.saveNode(updatedNodeData);
		} catch (error) {
			console.error(`[NodeStore - updateNodeAttributes] Error saving node ${nodeId} after attribute update:`, error);
			// Revert store update on save failure?
			nodes.update(n => n.set(nodeId, dataNode)); // Example revert
		}
	} else {
		console.warn("[NodeStore - updateNodeAttributes] LocalAdapter not initialized, persistence disabled.");
	}
}