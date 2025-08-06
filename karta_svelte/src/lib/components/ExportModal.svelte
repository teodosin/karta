<script lang="ts">
  import { fade, scale, slide } from 'svelte/transition';
  import { selectedNodes, selectedCount, exportActions } from '$lib/karta/ExportStore';
  import type { ExportableNode } from '$lib/karta/ExportStore';
  import type { BundleTreeResponse, ExportBundleRequest } from '$lib/types/types';
  import { Package, Eye, Folder, FolderOpen, File, FileText, List, GitBranch, Info } from 'lucide-svelte';
  import { ServerAdapter } from '$lib/util/ServerAdapter';
  
  export let isOpen = false;
  
  let exportTitle = '';
  let exportDescription = '';
  let includeAssets = true;
  let modalElement: HTMLDivElement;
  let showTreeView = false;
  let bundleTree: BundleTreeResponse | null = null;
  
  const serverAdapter = new ServerAdapter();
  
  $: stats = exportActions.getExportStats();
  
  function closeModal() {
    isOpen = false;
  }
  
  function removeNode(nodeId: string) {
    exportActions.removeNode(nodeId);
  }
  
  function clearAll() {
    exportActions.clearSelection();
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      closeModal();
    }
  }

  // Get detailed tree structure for bundle preview
  async function loadBundleTree() {
    if (bundleTree) {
      showTreeView = !showTreeView;
      return;
    }
    
    try {
      const nodeIds = exportActions.getExportableNodeIds();
      bundleTree = await serverAdapter.getBundleTree(nodeIds);
      showTreeView = true;
    } catch (error) {
      console.error('Failed to load bundle tree:', error);
      // Fall back to mock structure
      bundleTree = {
        tree: {
          name: "Export Bundle",
          path: "/",
          node_type: "bundle",
          is_directory: true,
          children: $selectedNodes.map(node => ({
            name: getNodeDisplayName(node),
            path: node.node.path || node.id,
            node_type: node.node.ntype,
            is_directory: node.node.ntype === 'core/fs/dir',
            children: node.includeChildren ? [] : undefined
          }))
        },
        total_files: $selectedNodes.length,
        total_size: 0,
        includes_assets: true
      };
      showTreeView = true;
    }
  }

  // TODO: Implement preview functionality using Tauri
  // This will open a new Tauri window with the runtime component displaying the selected bundle
  // The runtime will be read-only and show exactly what the exported bundle contains
  async function previewBundle() {
    try {
      // TODO: When runtime refactor is complete, implement this:
      // 1. Create a temporary bundle from selected nodes
      // 2. Use Tauri's window API to open a new window
      // 3. Load the runtime component in the new window
      // 4. Pass the bundle data to the runtime for preview
      
      console.log('Preview functionality will be implemented after runtime refactor');
      console.log('Selected nodes for preview:', exportActions.getExportableNodeIds());
      
      // Placeholder: For now, just show an alert
      alert(`Preview functionality coming soon!\n\nWould preview ${$selectedCount} selected items in a new runtime window.`);
      
      /* Future implementation:
      const { invoke } = await import('@tauri-apps/api/tauri');
      const { WebviewWindow } = await import('@tauri-apps/api/window');
      
      // Create bundle data
      const bundleData = {
        title: exportTitle || 'Preview Bundle',
        description: exportDescription,
        nodes: exportActions.getExportableNodeIds(),
        includeAssets
      };
      
      // Open new Tauri window with runtime
      const previewWindow = new WebviewWindow('bundle-preview', {
        url: '/runtime-preview',
        title: `Preview: ${bundleData.title}`,
        width: 1200,
        height: 800,
        resizable: true
      });
      
      // Pass bundle data to the preview window
      await previewWindow.emit('load-bundle', bundleData);
      */
    } catch (error) {
      console.error('Preview failed:', error);
    }
  }
  
  async function requestExport() {
    try {
      const exportRequest: ExportBundleRequest = {
        node_ids: exportActions.getExportableNodeIds(),
        title: exportTitle || undefined,
        description: exportDescription || undefined,
        include_assets: includeAssets
      };
      
      console.log('Requesting export with:', exportRequest);
      
      const response = await serverAdapter.exportBundle(exportRequest);
      
      // Open download link
      const downloadUrl = serverAdapter.downloadBundle(response.bundle_id);
      window.open(downloadUrl, '_blank');
      
      closeModal();
    } catch (error) {
      console.error('Export failed:', error);
      alert('Export failed. Please try again.');
    }
  }
  
  function formatDate(date: Date): string {
    return date.toLocaleString();
  }
  
  function getNodeIcon(node: ExportableNode): string {
    // Return component name for Lucide icons instead of emoji
    if (node.node.ntype === 'core/fs/dir') {
      return node.includeChildren ? 'FolderOpen' : 'Folder';
    }
    return 'File';
  }
  
  function getNodeIconComponent(node: ExportableNode) {
    if (node.node.ntype === 'core/fs/dir') {
      return node.includeChildren ? FolderOpen : Folder;
    }
    return File;
  }
  
  function getNodeDisplayName(node: ExportableNode): string {
    return node.node.attributes?.name as string || node.node.path || node.id;
  }
</script>

{#if isOpen}
  <!-- Transparent backdrop for click-to-close -->
  <div 
    class="fixed inset-0 z-40"
    transition:fade={{ duration: 200 }}
    on:click={closeModal}
    role="button"
    tabindex="-1"
    on:keydown={handleKeyDown}
  ></div>

  <!-- Modal content -->
  <div
    bind:this={modalElement}
    class="fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 z-50 
           border border-gray-700 rounded-lg shadow-2xl w-[700px] max-w-[90vw] h-[500px] flex flex-col"
    style="background-color: color-mix(in srgb, var(--color-panel-bg) 80%, transparent);"
    role="dialog"
    aria-modal="true"
    aria-labelledby="export-modal-title"
    transition:scale={{ duration: 200, start: 0.95 }}
    on:keydown={handleKeyDown}
    tabindex="-1"
  >
    <!-- Modal header -->
    <div class="p-3 border-b border-gray-700">
      <label id="export-modal-title" class="text-sm font-medium" style="color: var(--color-text-color);">
        Export Bundle ({$selectedCount} {$selectedCount === 1 ? 'item' : 'items'})
      </label>
      
      <!-- Export settings -->
      <div class="mt-3 space-y-3">
        <div>
          <input
            id="export-title"
            type="text"
            bind:value={exportTitle}
            placeholder="Bundle title (optional)"
            autocomplete="off"
            class="w-full px-3 py-2 text-sm border border-gray-700 rounded 
                   text-white placeholder-gray-400
                   focus:outline-none focus:ring-1 focus:border-transparent"
            style="background-color: var(--color-viewport-bg); --tw-ring-color: var(--color-contrast-color);"
            on:focus={(e) => e.currentTarget.style.borderColor = 'var(--color-contrast-color)'}
            on:blur={(e) => e.currentTarget.style.borderColor = 'transparent'}
          />
        </div>
        
        <div>
          <textarea
            id="export-description"
            bind:value={exportDescription}
            placeholder="Description (optional)"
            rows="2"
            autocomplete="off"
            class="w-full px-3 py-2 text-sm border border-gray-700 rounded 
                   text-white placeholder-gray-400 resize-none
                   focus:outline-none focus:ring-1 focus:border-transparent"
            style="background-color: var(--color-viewport-bg); --tw-ring-color: var(--color-contrast-color);"
            on:focus={(e) => e.currentTarget.style.borderColor = 'var(--color-contrast-color)'}
            on:blur={(e) => e.currentTarget.style.borderColor = 'transparent'}
          ></textarea>
        </div>
        
        <div class="flex items-center justify-between">
          <label class="flex items-center text-sm" style="color: var(--color-text-color);">
            <input
              id="include-assets"
              type="checkbox"
              bind:checked={includeAssets}
              class="h-4 w-4 mr-2 accent-blue-600"
            />
            Include assets (images, files)
          </label>
          
          <!-- Stats -->
          <div class="text-xs opacity-70 flex items-center gap-2" style="color: var(--color-text-color);">
            <div class="flex items-center gap-1">
              <File size={12} />
              {stats.files} files
            </div>
            <div class="flex items-center gap-1">
              <Folder size={12} />
              {stats.directories} directories
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Selected items list -->
    <div class="flex-1 overflow-hidden flex flex-col">
      {#if $selectedNodes.length === 0}
        <div class="p-6 text-center">
          <div class="text-gray-400">
            <div class="flex justify-center mb-2">
              <Package size={32} />
            </div>
            <div class="text-sm font-medium mb-1">No items selected</div>
            <div class="text-xs">Right-click on nodes or directories to add them to your export bundle</div>
          </div>
        </div>
      {:else}
        <!-- View toggle header -->
        <div class="p-3 border-b border-gray-700 flex items-center justify-between">
          <div class="text-sm font-medium" style="color: var(--color-text-color);">
            Selected Items
          </div>
          <button
            on:click={loadBundleTree}
            class="text-xs px-2 py-1 rounded hover:bg-gray-600 flex items-center gap-1"
            style="color: var(--color-text-color);"
          >
            <svelte:component this={showTreeView ? List : GitBranch} size={12} />
            {showTreeView ? 'List View' : 'Tree View'}
          </button>
        </div>

        <div class="flex-1 overflow-y-auto">
          {#if showTreeView && bundleTree}
            <!-- Tree structure view -->
            <div class="p-3">
              <div class="text-xs font-medium mb-2" style="color: var(--color-text-color);">
                Bundle Structure Preview
              </div>
              <!-- Tree component with server data -->
              <div class="space-y-1 text-xs" style="color: var(--color-text-color);">
                <div class="font-mono flex items-center gap-1">
                  <Package size={12} />
                  {bundleTree.tree.name}
                </div>
                {#if bundleTree.tree.children}
                  {#each bundleTree.tree.children as child}
                    <div class="pl-4 font-mono flex items-center gap-1">
                      <svelte:component this={child.is_directory ? Folder : File} size={12} />
                      {child.name}
                      {#if child.children}
                        {#each child.children as subChild}
                          <div class="pl-8 text-gray-400 flex items-center gap-1">
                            <svelte:component this={subChild.is_directory ? Folder : File} size={10} />
                            {subChild.name}
                          </div>
                        {/each}
                      {/if}
                    </div>
                  {/each}
                {/if}
              </div>
              <div class="mt-3 p-2 bg-gray-700/50 rounded text-xs text-gray-400 flex items-center gap-2">
                <Info size={12} />
                {bundleTree.total_files} files • {Math.round(bundleTree.total_size / 1024)}KB total
              </div>
            </div>
          {:else}
            <!-- Simple list view -->
            <ul class="h-full" role="listbox">
              {#each $selectedNodes as exportableNode (exportableNode.id)}
                <li
                  class="px-3 py-2 text-xs border-b border-gray-700 last:border-b-0 flex items-center justify-between"
                  style="color: var(--color-text-color);"
                  transition:slide={{ duration: 200 }}
                >
                  <div class="flex items-center space-x-3 flex-1 min-w-0">
                <svelte:component this={getNodeIconComponent(exportableNode)} size={16} />
                <div class="flex-1 min-w-0">
                      <div class="font-mono text-sm truncate">
                        {getNodeDisplayName(exportableNode)}
                      </div>
                      <div class="text-xs opacity-60">
                        Added {formatDate(exportableNode.addedAt)}
                        {#if exportableNode.includeChildren}
                          • Includes subdirectories
                        {/if}
                      </div>
                    </div>
                  </div>
                  <button
                    on:click={() => removeNode(exportableNode.id)}
                    class="text-red-400 hover:text-red-300 text-xs ml-3 px-2 py-1 rounded hover:bg-red-900/20"
                    aria-label="Remove from export"
                  >
                    Remove
                  </button>
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      {/if}
    </div>

    <!-- Modal footer -->
    <div class="p-2 border-t border-gray-700" style="background-color: color-mix(in srgb, var(--color-panel-bg) 80%, black);">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          {#if $selectedCount > 0}
            <button
              on:click={clearAll}
              class="text-xs text-red-400 hover:text-red-300"
            >
              Clear All
            </button>
          {/if}
        </div>
        
        <div class="flex items-center gap-2">
          <button
            on:click={closeModal}
            class="px-3 py-1.5 text-xs font-medium rounded"
            style="color: var(--color-text-color); background-color: var(--color-viewport-bg);"
          >
            Cancel
          </button>
          <button
            on:click={previewBundle}
            disabled={$selectedCount === 0}
            class="px-3 py-1.5 text-xs font-medium rounded bg-gray-600 hover:bg-gray-500 disabled:bg-gray-700 disabled:cursor-not-allowed text-white flex items-center gap-1"
            title="Preview bundle in runtime (coming soon)"
          >
            <Eye size={12} />
            Preview
          </button>
          <button
            on:click={requestExport}
            disabled={$selectedCount === 0}
            class="px-3 py-1.5 text-xs font-medium text-white bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 disabled:cursor-not-allowed rounded"
          >
            Export Bundle
          </button>
        </div>
      </div>
      
      <!-- Keyboard shortcuts -->
      <div class="flex items-center justify-center text-xs text-gray-400 mt-2">
        <span><kbd class="px-1 py-0.5 bg-gray-700 rounded text-xs">Esc</kbd> Close</span>
      </div>
    </div>
  </div>
{/if}

<style>
  /* Custom scrollbar for export list */
  ul::-webkit-scrollbar {
    width: 6px;
  }

  ul::-webkit-scrollbar-track {
    background: transparent;
  }

  ul::-webkit-scrollbar-thumb {
    background-color: rgba(156, 163, 175, 0.5);
    border-radius: 3px;
  }

  ul::-webkit-scrollbar-thumb:hover {
    background-color: rgba(107, 114, 128, 0.7);
  }

  :global(.dark) ul::-webkit-scrollbar-thumb {
    background-color: rgba(107, 114, 128, 0.5);
  }

  :global(.dark) ul::-webkit-scrollbar-thumb:hover {
    background-color: rgba(156, 163, 175, 0.7);
  }

  /* Keyboard shortcuts styling */
  kbd {
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
    font-size: 0.75rem;
    font-weight: 500;
  }
</style>
