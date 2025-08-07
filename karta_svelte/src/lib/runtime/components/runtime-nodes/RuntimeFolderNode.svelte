<script context="module" lang="ts">
    import type { RuntimeNodeDefinition } from '../../stores/RuntimeNodeRegistry';
    
    export const runtimeNodeDef: RuntimeNodeDefinition = {
        ntype: 'core/folder',
        displayName: 'Folder'
    };
</script>

<script lang="ts">
    import type { DataNode, ViewNode } from '../../../types/types';
    
    export let dataNode: DataNode;
    export let viewNode: ViewNode; // External reference
    export let width: number;
    export let height: number;
    
    // Get display content
    $: folderName = getNameFromPath(dataNode.path) || dataNode.attributes?.name || 'Untitled Folder';
    $: folderPath = dataNode.path;
    
    function getNameFromPath(path: string): string {
        if (!path) return '';
        const segments = path.split('/');
        return segments[segments.length - 1] || '';
    }
</script>

<div 
    class="folder-node"
    style="width: {width}px; height: {height}px;"
>
    <div class="content">
        <div class="folder-icon">📁</div>
        <div class="folder-info">
            <div class="folder-name">{folderName}</div>
            {#if folderPath && folderPath !== folderName}
                <div class="folder-path">{folderPath}</div>
            {/if}
        </div>
    </div>
</div>

<style>
    .folder-node {
        border: 2px solid #8b5cf6;
        background: #f3e8ff;
        border-radius: 8px;
        padding: 12px;
        box-sizing: border-box;
        display: flex;
        align-items: center;
        justify-content: center;
        overflow: hidden;
    }
    
    .content {
        width: 100%;
        height: 100%;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        text-align: center;
        gap: 6px;
    }
    
    .folder-icon {
        font-size: 28px;
        margin-bottom: 4px;
    }
    
    .folder-info {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 2px;
    }
    
    .folder-name {
        font-weight: 600;
        font-size: 13px;
        color: #6b21a8;
        word-break: break-word;
        line-height: 1.3;
    }
    
    .folder-path {
        font-size: 9px;
        color: #7c3aed;
        font-family: monospace;
        word-break: break-all;
        line-height: 1.2;
        max-height: 2.4em;
        overflow: hidden;
        opacity: 0.8;
    }
</style>
