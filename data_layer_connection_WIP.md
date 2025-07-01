diff --git a/src/lib/karta/NodeStore.ts b/src/lib/karta/NodeStore.ts
index 69e7ab4..c72cc99 100644
--- a/src/lib/karta/NodeStore.ts
+++ b/src/lib/karta/NodeStore.ts
@@ -2,6 +2,7 @@ import { writable, get } from 'svelte/store';
 import { setSelectedNodes } from './SelectionStore'; // Import selection action
 // Removed highlightNodeId import
 import { localAdapter } from '../util/LocalAdapter';
+import { ServerAdapter } from '../util/ServerAdapter';
 import type { DataNode, NodeId, AssetData, AbsoluteTransform, ViewportSettings, TweenableNodeState, StorableContext, StorableViewNode, StorableViewportSettings, ViewNode, Context } from '../types/types'; // Added Context type
 import { v4 as uuidv4 } from 'uuid';
 import { getDefaultAttributesForType, getDefaultViewNodeStateForType } from '$lib/node_types/registry';
@@ -12,13 +13,16 @@ import { viewTransform, centerViewOnCanvasPoint } from './ViewportStore'; // Imp
 
 export const nodes = writable<Map<NodeId, DataNode>>(new Map());
 
+// Use a generic persistence service, which can be LocalAdapter or ServerAdapter
+const persistenceService = new ServerAdapter(); // Or dynamically switch based on config
+
 async function _ensureDataNodeExists(nodeId: NodeId): Promise<DataNode | null> {
-    if (!localAdapter) {
-        console.error(`[_ensureDataNodeExists] LocalAdapter not available while checking for ${nodeId}`);
+    if (!persistenceService) {
+        console.error(`[_ensureDataNodeExists] Persistence service not available while checking for ${nodeId}`);
         return null;
     }
     try {
-        let dataNode = await localAdapter.getNode(nodeId);
+        let dataNode = await persistenceService.getNode(nodeId);
         if (nodeId === ROOT_NODE_ID) {
         }
         if (!dataNode) {
@@ -40,7 +44,9 @@ async function _ensureDataNodeExists(nodeId: NodeId): Promise<DataNode | null> {
                 path: defaultPath, // Set path correctly
                 attributes: { name: defaultName, ...(isRoot && { isSystemNode: true, view_isNameVisible: false }) }, // Set name, system flag, and hide label for root
             };
-            await localAdapter.saveNode(dataNode);
+            // This initial save might need to be adapted for ServerAdapter if it requires a parent path
+            // For now, we assume it can handle a simple save.
+            await persistenceService.saveNode(dataNode);
         }
         return dataNode;
     } catch (error) {
@@ -59,26 +65,20 @@ export async function createNodeAtPosition(
 ): Promise<NodeId | null> {
 	const newNodeId: NodeId = uuidv4();
     const now = Date.now();
-    let baseName = attributes.name || ntype;
-    let finalName = baseName;
-    let counter = 2;
-    // Ensure name uniqueness (assuming checkNameExists works correctly)
-    if (localAdapter) {
-        while (await localAdapter.checkNameExists(finalName)) {
-            finalName = `${baseName}${counter}`; counter++;
-        }
-    } else { console.warn("LocalAdapter not ready, cannot check for duplicate names."); }
-
-	// 1. Create DataNode
-	const newNodeData: DataNode = {
-		id: newNodeId, ntype: ntype, createdAt: now, modifiedAt: now,
-        path: `/${finalName}`, // Simple path for now
-		attributes: {
-		          ...getDefaultAttributesForType(ntype), // Get new prefixed defaults
-		          ...attributes, // Apply user-provided attributes (can override defaults)
-		          name: finalName // Ensure name is set (unprefixed)
-		      },
-	};
+    const baseName = attributes.name || ntype;
+
+ // 1. Create DataNode
+    // The client no longer determines the finalName or path. It sends the desired name,
+    // and the server returns the authoritative state.
+ const newNodeData: DataNode = {
+  id: newNodeId, ntype: ntype, createdAt: now, modifiedAt: now,
+        path: `/${baseName}`, // Path will be corrected by the server response.
+  attributes: {
+            ...getDefaultAttributesForType(ntype),
+            ...attributes,
+            name: baseName
+        },
+ };
 
     // 2. Get default view state based on ntype and create initial state for the ViewNode's tween
     const defaultViewState = getDefaultViewNodeStateForType(ntype); // Gets { width, height, scale, rotation }
@@ -100,36 +100,53 @@ export async function createNodeAtPosition(
 
     // 4. Update stores
     const contextId = get(currentContextId);
-	nodes.update(n => n.set(newNodeId, newNodeData)); // Add DataNode
-
-    contexts.update((ctxMap: Map<NodeId, Context>) => { // Explicitly type ctxMap
-        let currentCtx = ctxMap.get(contextId);
-        if (!currentCtx) {
-            console.warn(`Context ${contextId} not found when creating node ${newNodeId}. Creating context.`);
-            currentCtx = { id: contextId, viewNodes: new Map() };
-            ctxMap.set(contextId, currentCtx); // Add new context to map if needed
-        }
-        currentCtx.viewNodes.set(newNodeId, newViewNode); // Add new ViewNode to context
-        return ctxMap; // Return the modified map to trigger update
-    });
+    // We will add the node to the store *after* it has been successfully created on the server.
+    // This prevents race conditions where updates are sent for a node that doesn't exist yet.
 
     // 5. Persist changes
-    if (localAdapter) {
+    if (persistenceService) {
         try {
-            await localAdapter.saveNode(newNodeData);
+            // For ServerAdapter, we need the parent path.
+            const parentNode = get(nodes).get(contextId);
+            const parentPath = parentNode?.path || '/'; // Fallback to root
+            
+            const persistedNode = await (persistenceService as ServerAdapter).createNode(newNodeData, parentPath);
+
+            if (!persistedNode) {
+                throw new Error("Node creation failed on the server.");
+            }
+
+            // NOW, update the stores with the definitive node data from the server
+            nodes.update(n => n.set(persistedNode.id, persistedNode)); // Add the persisted DataNode
+            
+            // CRITICAL FIX: The ViewNode was created with a temporary client-side ID.
+            // We need to update the contexts store to use the server-authoritative ID.
+            contexts.update((ctxMap: Map<NodeId, Context>) => {
+                let currentCtx = ctxMap.get(contextId);
+                if (!currentCtx) {
+                    console.warn(`Context ${contextId} not found when creating node ${persistedNode.id}. Creating context.`);
+                    currentCtx = { id: contextId, viewNodes: new Map() };
+                    ctxMap.set(contextId, currentCtx);
+                }
+                // Remove the old ViewNode with the temporary ID and add it back with the correct ID.
+                currentCtx.viewNodes.delete(newNodeId);
+                newViewNode.id = persistedNode.id; // Update the ViewNode's ID
+                currentCtx.viewNodes.set(persistedNode.id, newViewNode);
+                return ctxMap;
+            });
+            
             const updatedCtx = get(contexts).get(contextId);
             if (updatedCtx) {
-                updatedCtx.viewportSettings = { ...viewTransform.current }; // Capture viewport state - Access .current directly
-                await localAdapter.saveContext(updatedCtx); // Save context
+                updatedCtx.viewportSettings = { ...viewTransform.current };
+                await persistenceService.saveContext(updatedCtx);
             }
-            return newNodeId; // Return ID on successful save
+            return persistedNode.id; // Return ID on successful save
         } catch (error) {
             console.error("Error saving node or context after creation:", error);
-            // Optionally remove the node from stores if save failed? For now, just return null.
             return null;
         }
     } else {
-        console.warn("LocalAdapter not initialized, persistence disabled.");
+        console.warn("Persistence service not initialized, persistence disabled.");
         // Still add to stores for non-persistent use, but return ID
         return newNodeId;
         // Or return null if persistence is strictly required? Let's return ID for now.
@@ -180,8 +197,8 @@ export async function createImageNodeWithAsset(
     initialWidth?: number,
     initialHeight?: number
 ): Promise<NodeId | null> {
-    if (!localAdapter) {
-        console.error("[createImageNodeWithAsset] LocalAdapter not available.");
+    if (!persistenceService) {
+        console.error("[createImageNodeWithAsset] Persistence service not available.");
         URL.revokeObjectURL(objectUrl); // Revoke if we can't save
         return null;
     }
@@ -205,7 +222,7 @@ export async function createImageNodeWithAsset(
 
         // 2. Prepare asset data and save it using the newNodeId as assetId
         const assetData: AssetData = { blob: imageBlob, mimeType: imageBlob.type, name: assetName };
-        await localAdapter.saveAsset(newNodeId, assetData);
+        await persistenceService.saveAsset(newNodeId, assetData);
 
         // 3. Update the node attributes with the Object URL and assetId
         // Use the existing updateNodeAttributes function
@@ -218,9 +235,9 @@ export async function createImageNodeWithAsset(
         // Cleanup logic:
         if (newNodeId) {
             // If the base node was created, attempt to delete it from DB.
-            // The updated localAdapter.deleteNode will also call deleteAsset,
+            // The updated persistenceService.deleteNode will also call deleteAsset,
             // which handles revoking any URL *it* might have tracked.
-            await localAdapter.deleteNode(newNodeId);
+            await persistenceService.deleteNode(newNodeId);
             // Also remove from stores manually
             nodes.update(n => { n.delete(newNodeId!); return n; });
             contexts.update((ctxMap: Map<NodeId, Context>) => { // Explicitly type ctxMap
@@ -255,43 +272,21 @@ export async function updateNodeAttributes(nodeId: NodeId, newAttributes: Record
     const oldName = dataNode.attributes?.name;
     const newName = newAttributes?.name;
     let attributesToSave = { ...dataNode.attributes, ...newAttributes }; // Merge old and new
-    // Check for name change and uniqueness
-    if (newName && newName.trim() && newName !== oldName) {
-        const finalNewName = newName.trim();
-        if (localAdapter) {
-            let nameToSet = finalNewName; // Start with the user's desired name
-            let nameExists = await localAdapter.checkNameExists(nameToSet);
-
-            if (nameExists) {
-                console.warn(`[updateNodeAttributes] Name "${nameToSet}" already exists. Finding next available name...`);
-                const baseName = nameToSet; // Store the original desired name
-                let counter = 2;
-                // Loop until a unique name is found
-                while (nameExists) {
-                    nameToSet = `${baseName}${counter}`;
-                    nameExists = await localAdapter.checkNameExists(nameToSet);
-                    counter++;
-                }
-            }
-            // Update attributesToSave with the final unique name
-            attributesToSave.name = nameToSet;
-        } else {
-            console.warn("[updateNodeAttributes] LocalAdapter not ready, cannot check for duplicate names.");
-            // If adapter isn't ready, we cannot guarantee uniqueness. Cancel the rename.
-            console.error("[updateNodeAttributes] LocalAdapter not available. Cannot verify name uniqueness. Rename cancelled.");
-            return;
-        }
-    } else if (newName !== undefined && !newName.trim()) {
+    // The client no longer handles name collision logic on update.
+    // It sends the desired attributes, and the server is responsible for any validation.
+    if (newAttributes.name !== undefined && !newAttributes.name.trim()) {
         console.warn(`[updateNodeAttributes] Attempted to rename node ${nodeId} to an empty name. Operation cancelled.`);
         return; // Prevent empty names
     }
 
     // Create updated node data
     const updatedNodeData: DataNode = {
-        ...dataNode,
+        id: dataNode.id,
+        ntype: dataNode.ntype,
+        createdAt: dataNode.createdAt,
+        path: `/${attributesToSave.name}`, // Update the path based on the new name
         attributes: attributesToSave, // Use the final attributes map (name might have been incremented)
         modifiedAt: Date.now(),
-        path: dataNode.path // Do not change the path here. Path changes should be handled by a dedicated rename/move operation.
     };
 
     // Update the store
@@ -301,16 +296,16 @@ export async function updateNodeAttributes(nodeId: NodeId, newAttributes: Record
         nodes.update(n => n.set(nodeId, updatedNodeData));
 
         // Persist changes
-        if (localAdapter) {
+        if (persistenceService) {
             try {
-                await localAdapter.saveNode(updatedNodeData);
+                await (persistenceService as ServerAdapter).updateNode(updatedNodeData);
             } catch (error) {
                 console.error(`[updateNodeAttributes] Error saving node ${nodeId} after attribute update:`, error);
                 // Optionally revert store update on save failure?
                 // nodes.update(n => n.set(nodeId, dataNode)); // Example revert
             }
         } else {
-            console.warn("[updateNodeAttributes] LocalAdapter not initialized, persistence disabled.");
+            console.warn("[updateNodeAttributes] Persistence service not initialized, persistence disabled.");
         }
     } else {
     }
@@ -342,32 +337,32 @@ export async function updateNodeSearchableFlag(nodeId: NodeId, isSearchable: boo
     nodes.update(n => n.set(nodeId, updatedNodeData));
 
     // Persist changes
-    if (localAdapter) {
+    if (persistenceService) {
         try {
-            await localAdapter.saveNode(updatedNodeData);
+            await (persistenceService as ServerAdapter).updateNode(updatedNodeData);
         } catch (error) {
             console.error(`[updateNodeSearchableFlag] Error saving node ${nodeId} after searchable flag update:`, error);
             // Optionally revert store update on save failure?
             // nodes.update(n => n.set(nodeId, dataNode)); // Example revert
         }
     } else {
-        console.warn("[updateNodeSearchableFlag] LocalAdapter not initialized, persistence disabled.");
+        console.warn("[updateNodeSearchableFlag] Persistence service not initialized, persistence disabled.");
     }
 }
 
 
 export async function fetchAvailableContextDetails(): Promise<{ id: NodeId, name: string, path: string }[]> {
-    if (!localAdapter) {
-        console.error("[fetchAvailableContextDetails] LocalAdapter not available.");
+    if (!persistenceService) {
+        console.error("[fetchAvailableContextDetails] Persistence service not available.");
         return [];
     }
     try {
-        const contextIds = await localAdapter.getAllContextIds();
+        const contextIds = await persistenceService.getAllContextIds();
         if (contextIds.length === 0) {
             return [];
         }
 
-        const dataNodesMap = await localAdapter.getDataNodesByIds(contextIds);
+        const dataNodesMap = await persistenceService.getDataNodesByIds(contextIds);
         // Add type assertion for iterator
         const contextDetails = Array.from(dataNodesMap.values() as IterableIterator<DataNode>)
         	.map((node: DataNode) => ({ // Explicitly type node
@@ -389,8 +384,8 @@ export async function fetchAvailableContextDetails(): Promise<{ id: NodeId, name
    import { edges, deleteEdge } from './EdgeStore'; // Assuming EdgeStore exports edges store and deleteEdge action
    
    export async function deleteDataNodePermanently(nodeId: NodeId): Promise<void> {
-    if (!localAdapter) {
-    	console.error("[deleteDataNodePermanently] LocalAdapter not available.");
+    if (!persistenceService) {
+    	console.error("[deleteDataNodePermanently] Persistence service not available.");
     	return;
     }
    
@@ -399,7 +394,7 @@ export async function fetchAvailableContextDetails(): Promise<{ id: NodeId, name
     	const dataNodeToDelete = get(nodes).get(nodeId); // Get from store first
     	if (!dataNodeToDelete) {
     		// If not in store, try fetching from DB (might be a ghost node scenario)
-    		const nodeFromDb = await localAdapter.getNode(nodeId);
+    		const nodeFromDb = await persistenceService.getNode(nodeId);
     		if (!nodeFromDb) {
     			console.warn(`[deleteDataNodePermanently] Node ${nodeId} not found in store or DB. Cannot delete.`);
     			return;
@@ -433,18 +428,18 @@ export async function fetchAvailableContextDetails(): Promise<{ id: NodeId, name
    
     	// 5. Delete asset if it's an image node
     	// Use the node data we fetched earlier (either from store or DB)
-    	const nodeType = dataNodeToDelete?.ntype ?? (await localAdapter.getNode(nodeId))?.ntype; // Check type
+    	const nodeType = dataNodeToDelete?.ntype ?? (await persistenceService.getNode(nodeId))?.ntype; // Check type
     	if (nodeType === 'core/image') {
     		// Asset ID is the same as Node ID for images currently
-    		await localAdapter.deleteAsset(nodeId);
+    		await persistenceService.deleteAsset(nodeId);
     	}
    
     	// 6. Delete the DataNode from persistence
-    	await localAdapter.deleteNode(nodeId);
+    	await persistenceService.deleteNode(nodeId);
     
     	// 7. Delete the corresponding Context (if it exists) and update the map
     	try {
-    		await localAdapter.deleteContext(nodeId);
+    		await persistenceService.deleteContext(nodeId);
     		// If context deletion was successful, remove from the map
     		availableContextsMap.update(map => {
     			if (map.has(nodeId)) {
diff --git a/src/lib/util/ServerAdapter.ts b/src/lib/util/ServerAdapter.ts
index a93183f..5fe97ac 100644
--- a/src/lib/util/ServerAdapter.ts
+++ b/src/lib/util/ServerAdapter.ts
@@ -88,10 +88,122 @@ function transformServerAttributesToRecord(serverAttributes: ServerAttribute[]):
     return record;
 }
 
+/**
+ * Transforms a client-side DataNode's attributes into the format expected by the server.
+ * @param attributes The client-side attributes record.
+ * @returns An array of ServerAttribute objects.
+ */
+function transformAttributesToServerFormat(attributes: Record<string, any>): ServerAttribute[] {
+    return Object.entries(attributes)
+        .filter(([key, value]) => value !== undefined && value !== null)
+        .map(([name, value]) => {
+            // The backend expects a tagged union format.
+            let taggedValue: any;
+            switch (typeof value) {
+                case 'string':
+                    taggedValue = { String: value };
+                    break;
+                case 'number':
+                    // Using Float as a general case for numbers.
+                    taggedValue = { Float: value };
+                    break;
+                case 'boolean':
+                    taggedValue = { UInt: value ? 1 : 0 };
+                    break;
+                default:
+                    // For complex objects, serialize them as a JSON string.
+                    taggedValue = { String: JSON.stringify(value) };
+                    break;
+            }
+            return { name, value: taggedValue };
+        });
+}
+
 
 export class ServerAdapter implements PersistenceService {
     constructor() {}
 
+    async createNode(node: DataNode, parentPath: string): Promise<DataNode | undefined> {
+        const url = `${SERVER_BASE_URL}/api/nodes`;
+        const payload = {
+            name: node.attributes['name'] || 'Unnamed Node',
+            ntype: { type_path: node.ntype, version: "0.1.0" },
+            parent_path: parentPath,
+            attributes: transformAttributesToServerFormat(node.attributes),
+        };
+
+        try {
+            const response = await fetch(url, {
+                method: 'POST',
+                headers: { 'Content-Type': 'application/json' },
+                body: JSON.stringify(payload),
+            });
+
+            if (!response.ok) {
+                const errorBody = await response.text();
+                console.error(`[ServerAdapter.createNode] Error creating node. Status: ${response.status}`, errorBody);
+                throw new Error(`Server responded with status ${response.status}`);
+            }
+
+            const serverNode: ServerDataNode = await response.json();
+            const attributes = transformServerAttributesToRecord(serverNode.attributes);
+            attributes['name'] = serverNode.name;
+
+            return {
+                id: serverNode.uuid,
+                ntype: serverNode.ntype.type_path,
+                createdAt: (serverNode.created_time?.secs_since_epoch ?? 0) * 1000,
+                modifiedAt: (serverNode.modified_time?.secs_since_epoch ?? 0) * 1000,
+                path: serverNode.path,
+                attributes: attributes,
+                isSearchable: attributes['isSearchable'] ?? true,
+            };
+
+        } catch (error) {
+            console.error(`[ServerAdapter.createNode] Network error creating node:`, error);
+            throw error;
+        }
+    }
+
+    async updateNode(node: DataNode): Promise<DataNode | undefined> {
+        const url = `${SERVER_BASE_URL}/api/nodes/${node.id}`;
+        const payload = {
+             attributes: transformAttributesToServerFormat(node.attributes),
+        };
+
+        try {
+            const response = await fetch(url, {
+                method: 'PUT',
+                headers: { 'Content-Type': 'application/json' },
+                body: JSON.stringify(payload),
+            });
+
+            if (!response.ok) {
+                const errorBody = await response.text();
+                console.error(`[ServerAdapter.updateNode] Error updating node ${node.id}. Status: ${response.status}`, errorBody);
+                throw new Error(`Server responded with status ${response.status}`);
+            }
+            
+            const serverNode: ServerDataNode = await response.json();
+            const attributes = transformServerAttributesToRecord(serverNode.attributes);
+            attributes['name'] = serverNode.name;
+
+            return {
+                id: serverNode.uuid,
+                ntype: serverNode.ntype.type_path,
+                createdAt: (serverNode.created_time?.secs_since_epoch ?? 0) * 1000,
+                modifiedAt: (serverNode.modified_time?.secs_since_epoch ?? 0) * 1000,
+                path: serverNode.path,
+                attributes: attributes,
+                isSearchable: attributes['isSearchable'] ?? true,
+            };
+
+        } catch (error) {
+            console.error(`[ServerAdapter.updateNode] Network error updating node ${node.id}:`, error);
+            throw error;
+        }
+    }
+
     async loadContextBundle(contextPath: string): Promise<ContextBundle | undefined> {
         const encodedPath = encodeURIComponent(contextPath);
         const url = `${SERVER_BASE_URL}/ctx/${encodedPath}`;
@@ -219,26 +331,7 @@ export class ServerAdapter implements PersistenceService {
             const isNameVisible = attributes?.['isNameVisible'] ?? true;
 
             const serverAttributes: ServerAttribute[] = attributes
-                ? Object.entries(attributes)
-                      .filter(([key, value]) => value !== undefined && value !== null && key !== 'isNameVisible')
-                      .map(([name, value]) => {
-                          let taggedValue: any;
-                          switch (typeof value) {
-                              case 'string':
-                                  taggedValue = { String: value };
-                                  break;
-                              case 'number':
-                                  taggedValue = { Float: value };
-                                  break;
-                              case 'boolean':
-                                  taggedValue = { UInt: value ? 1 : 0 };
-                                  break;
-                              default:
-                                  taggedValue = { String: JSON.stringify(value) };
-                                  break;
-                          }
-                          return { name, value: taggedValue };
-                      })
+                ? transformAttributesToServerFormat(attributes)
                 : [];
 
             return {
@@ -295,7 +388,10 @@ export class ServerAdapter implements PersistenceService {
     }
 
     // --- Stubbed Methods ---
-    async saveNode(node: DataNode): Promise<void> { console.warn('[ServerAdapter.saveNode] Not implemented'); }
+    async saveNode(node: DataNode): Promise<void> {
+        // This can be a wrapper, but for now we expect the caller to use create/update directly.
+        console.warn('[ServerAdapter.saveNode] Deprecated. Use createNode or updateNode directly.');
+    }
     async getNode(nodeId: string): Promise<DataNode | undefined> { console.warn(`[ServerAdapter.getNode] Not implemented for ID: ${nodeId}`); return undefined; }
     async deleteNode(nodeId: string): Promise<void> { console.warn(`[ServerAdapter.deleteNode] Not implemented for ID: ${nodeId}`); }
     async getNodes(): Promise<DataNode[]> { console.warn('[ServerAdapter.getNodes] Not implemented'); return []; }
