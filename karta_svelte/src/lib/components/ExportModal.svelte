<script lang="ts">
  import { fade, slide } from 'svelte/transition';
  import { selectedNodes, selectedCount, exportActions } from '$lib/karta/ExportStore';
  import type { ExportableNode } from '$lib/karta/ExportStore';
  
  export let isOpen = false;
  
  let exportTitle = '';
  let exportDescription = '';
  let includeAssets = true;
  
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
  
  async function requestExport() {
    // TODO: Implement server-side export request
    const exportableNodeIds = exportActions.getExportableNodeIds();
    
    console.log('Requesting export with:', {
      nodeIds: exportableNodeIds,
      title: exportTitle || 'Karta Export',
      description: exportDescription,
      includeAssets
    });
    
    // For now, just close the modal
    // In the future, this will trigger server export and file download
    alert('Export functionality will be implemented server-side. Selected nodes: ' + exportableNodeIds.join(', '));
    closeModal();
  }
  
  function formatDate(date: Date): string {
    return date.toLocaleString();
  }
  
  function getNodeIcon(node: ExportableNode): string {
    if (node.node.ntype === 'core/fs/dir') {
      return node.includeChildren ? '📁' : '📂';
    }
    return '📄';
  }
  
  function getNodeDisplayName(node: ExportableNode): string {
    return node.node.attributes?.name as string || node.node.path || node.id;
  }
</script>

{#if isOpen}
  <!-- Modal backdrop -->
  <div 
    class="fixed inset-0 z-50 bg-black bg-opacity-50 flex items-center justify-center p-4"
    transition:fade={{ duration: 200 }}
    on:click={closeModal}
    role="dialog"
    aria-modal="true"
    aria-labelledby="export-modal-title"
  >
    <!-- Modal content -->
    <div 
      class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-2xl w-full max-h-[80vh] flex flex-col"
      transition:slide={{ duration: 300 }}
      on:click|stopPropagation
    >
      <!-- Header -->
      <div class="flex items-center justify-between p-6 border-b border-gray-200 dark:border-gray-600">
        <h2 id="export-modal-title" class="text-xl font-semibold text-gray-900 dark:text-gray-100">
          Export Bundle ({$selectedCount} {$selectedCount === 1 ? 'item' : 'items'})
        </h2>
        <button
          on:click={closeModal}
          class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 text-2xl leading-none"
          aria-label="Close modal"
        >
          ×
        </button>
      </div>
      
      <!-- Export settings -->
      <div class="p-6 border-b border-gray-200 dark:border-gray-600">
        <div class="space-y-4">
          <div>
            <label for="export-title" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Bundle Title
            </label>
            <input
              id="export-title"
              type="text"
              bind:value={exportTitle}
              placeholder="My Karta Export"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
            />
          </div>
          
          <div>
            <label for="export-description" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Description (optional)
            </label>
            <textarea
              id="export-description"
              bind:value={exportDescription}
              placeholder="Describe what this export contains..."
              rows="2"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 resize-none"
            ></textarea>
          </div>
          
          <div class="flex items-center">
            <input
              id="include-assets"
              type="checkbox"
              bind:checked={includeAssets}
              class="h-4 w-4 text-blue-600 border-gray-300 rounded"
            />
            <label for="include-assets" class="ml-2 text-sm text-gray-700 dark:text-gray-300">
              Include assets (images, files)
            </label>
          </div>
        </div>
        
        <!-- Stats -->
        <div class="mt-4 p-3 bg-gray-50 dark:bg-gray-700 rounded-md">
          <div class="text-sm text-gray-600 dark:text-gray-400">
            <div>📄 {stats.files} files</div>
            <div>📁 {stats.directories} directories {stats.hasRecursiveDirectories ? '(with contents)' : ''}</div>
          </div>
        </div>
      </div>
      
      <!-- Selected nodes list -->
      <div class="flex-1 overflow-hidden flex flex-col">
        <div class="p-4 border-b border-gray-200 dark:border-gray-600">
          <div class="flex items-center justify-between">
            <h3 class="text-lg font-medium text-gray-900 dark:text-gray-100">
              Selected Items
            </h3>
            {#if $selectedCount > 0}
              <button
                on:click={clearAll}
                class="text-sm text-red-600 hover:text-red-800 dark:text-red-400 dark:hover:text-red-300"
              >
                Clear All
              </button>
            {/if}
          </div>
        </div>
        
        <div class="flex-1 overflow-y-auto p-4">
          {#if $selectedNodes.length === 0}
            <div class="text-center text-gray-500 dark:text-gray-400 py-8">
              <div class="text-4xl mb-2">📦</div>
              <p>No items selected for export</p>
              <p class="text-sm mt-1">Right-click on nodes or directories to add them to your export bundle</p>
            </div>
          {:else}
            <div class="space-y-2">
              {#each $selectedNodes as exportableNode (exportableNode.id)}
                <div 
                  class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg"
                  transition:slide={{ duration: 200 }}
                >
                  <div class="flex items-center space-x-3 flex-1 min-w-0">
                    <span class="text-lg">{getNodeIcon(exportableNode)}</span>
                    <div class="flex-1 min-w-0">
                      <div class="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
                        {getNodeDisplayName(exportableNode)}
                      </div>
                      <div class="text-xs text-gray-500 dark:text-gray-400">
                        Added {formatDate(exportableNode.addedAt)}
                        {#if exportableNode.includeChildren}
                          • Includes subdirectories
                        {/if}
                      </div>
                    </div>
                  </div>
                  <button
                    on:click={() => removeNode(exportableNode.id)}
                    class="text-red-600 hover:text-red-800 dark:text-red-400 dark:hover:text-red-300 text-sm ml-3"
                    aria-label="Remove from export"
                  >
                    Remove
                  </button>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>
      
      <!-- Footer -->
      <div class="flex items-center justify-end space-x-3 p-6 border-t border-gray-200 dark:border-gray-600">
        <button
          on:click={closeModal}
          class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-600 hover:bg-gray-200 dark:hover:bg-gray-500 rounded-md"
        >
          Cancel
        </button>
        <button
          on:click={requestExport}
          disabled={$selectedCount === 0}
          class="px-4 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed rounded-md"
        >
          Export Bundle
        </button>
      </div>
    </div>
  </div>
{/if}
