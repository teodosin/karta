<script context="module" lang="ts">
    import type { RuntimeNodeDefinition } from '../../stores/RuntimeNodeRegistry';
    
    export const runtimeNodeDef: RuntimeNodeDefinition = {
        ntype: 'core/file',
        displayName: 'File'
    };
</script>

<script lang="ts">
    import type { DataNode, ViewNode } from '../../../types/types';
    
    export let dataNode: DataNode;
    export let viewNode: ViewNode; // External reference
    export let width: number;
    export let height: number;
    
    // Get display content
    $: fileName = getNameFromPath(dataNode.path) || dataNode.attributes?.name || 'Untitled File';
    $: filePath = dataNode.path;
    $: fileExtension = getFileExtension(fileName);
    
    function getNameFromPath(path: string): string {
        if (!path) return '';
        const segments = path.split('/');
        return segments[segments.length - 1] || '';
    }
    
    function getFileExtension(name: string): string {
        const parts = name.split('.');
        return parts.length > 1 ? parts[parts.length - 1].toLowerCase() : '';
    }
</script>

<div 
    class="file-node"
    style="width: {width}px; height: {height}px;"
>
    <div class="content">
        <div class="file-icon">📄</div>
        <div class="file-info">
            <div class="file-name">{fileName}</div>
            {#if fileExtension}
                <div class="file-ext">.{fileExtension}</div>
            {/if}
            {#if filePath && filePath !== fileName}
                <div class="file-path">{filePath}</div>
            {/if}
        </div>
    </div>
</div>

<style>
    .file-node {
        border: 2px solid #f59e0b;
        background: #fef3c7;
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
    
    .file-icon {
        font-size: 24px;
        margin-bottom: 4px;
    }
    
    .file-info {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 2px;
    }
    
    .file-name {
        font-weight: 600;
        font-size: 13px;
        color: #92400e;
        word-break: break-word;
        line-height: 1.3;
    }
    
    .file-ext {
        font-size: 11px;
        color: #a16207;
        font-weight: 500;
    }
    
    .file-path {
        font-size: 9px;
        color: #a16207;
        font-family: monospace;
        word-break: break-all;
        line-height: 1.2;
        max-height: 2.4em;
        overflow: hidden;
        opacity: 0.8;
    }
</style>
