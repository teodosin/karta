<script lang="ts">
  import { Folder, FolderOpen, File } from 'lucide-svelte';
  import type { BundleTreeNode } from '$lib/types/types';
  
  export let node: BundleTreeNode;
  export let depth: number = 0;
  
  let expanded = depth < 2; // Auto-expand first 2 levels
  
  function toggleExpanded() {
    expanded = !expanded;
  }
  
  function getNodeIcon() {
    if (node.is_directory) {
      return expanded ? FolderOpen : Folder;
    }
    return File;
  }
  
  function formatFileSize(size?: number): string {
    if (!size) return '';
    if (size < 1024) return `${size}B`;
    if (size < 1024 * 1024) return `${Math.round(size / 1024)}KB`;
    return `${Math.round(size / (1024 * 1024))}MB`;
  }
</script>

<div class="font-mono" style="padding-left: {depth * 16}px;">
  <div class="flex items-center gap-1 hover:bg-gray-700/30 px-1 py-0.5 rounded">
    {#if node.is_directory && node.children && node.children.length > 0}
      <button on:click={toggleExpanded} class="flex items-center">
        <svelte:component this={getNodeIcon()} size={12} />
      </button>
    {:else}
      <svelte:component this={getNodeIcon()} size={12} />
    {/if}
    
    <span class="flex-1">{node.name}</span>
    
    {#if node.size}
      <span class="text-xs text-gray-400">{formatFileSize(node.size)}</span>
    {:else if node.file_count}
      <span class="text-xs text-gray-400">{node.file_count} files</span>
    {/if}
  </div>
  
  {#if node.is_directory && expanded && node.children}
    {#each node.children as child}
      <svelte:self node={child} depth={depth + 1} />
    {/each}
  {/if}
</div>
