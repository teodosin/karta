<script context="module" lang="ts">
    import type { RuntimeNodeDefinition } from '../../stores/RuntimeNodeRegistry';
    
    export const runtimeNodeDef: RuntimeNodeDefinition = {
        ntype: 'core/generic',
        displayName: 'Generic Node'
    };
</script>

<script lang="ts">
    import type { DataNode, ViewNode } from '../../../types/types';
    
    export let dataNode: DataNode;
    export let viewNode: ViewNode; // External reference
    export let width: number;
    export let height: number;
    
    // Get display content
    $: displayName = dataNode.attributes?.name || getNameFromPath(dataNode.path) || dataNode.ntype;
    $: nodeType = dataNode.ntype;
    $: nodePath = dataNode.path;
    
    function getNameFromPath(path: string): string {
        if (!path) return '';
        const segments = path.split('/');
        return segments[segments.length - 1] || '';
    }
    
    // Color mapping for different node types
    $: borderColor = getTypeColor(nodeType);
    $: backgroundColor = getTypeBackgroundColor(nodeType);
    
    function getTypeColor(ntype: string): string {
        const colorMap: Record<string, string> = {
            'core/file': '#f59e0b',
            'core/folder': '#8b5cf6', 
            'core/image': '#10b981',
            'core/link': '#ef4444',
            'core/root': '#6b7280'
        };
        return colorMap[ntype] || '#6b7280';
    }
    
    function getTypeBackgroundColor(ntype: string): string {
        const bgMap: Record<string, string> = {
            'core/file': '#fef3c7',
            'core/folder': '#f3e8ff',
            'core/image': '#ecfdf5', 
            'core/link': '#fef2f2',
            'core/root': '#f9fafb'
        };
        return bgMap[ntype] || '#f9fafb';
    }
</script>

<div 
    class="generic-node"
    style="
        width: {width}px; 
        height: {height}px;
        border-color: {borderColor};
        background-color: {backgroundColor};
    "
>
    <div class="content">
        <div class="name">{displayName}</div>
        {#if nodePath && nodePath !== displayName}
            <div class="path">{nodePath}</div>
        {/if}
        <div class="type">{nodeType}</div>
    </div>
</div>

<style>
    .generic-node {
        border: 2px solid;
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
        gap: 4px;
    }
    
    .name {
        font-weight: 600;
        font-size: 14px;
        color: #374151;
        word-break: break-word;
        line-height: 1.3;
    }
    
    .path {
        font-size: 10px;
        color: #6b7280;
        font-family: monospace;
        word-break: break-all;
        line-height: 1.2;
        max-height: 2.4em;
        overflow: hidden;
    }
    
    .type {
        font-size: 10px;
        color: #9ca3af;
        font-family: monospace;
        margin-top: auto;
    }
</style>
