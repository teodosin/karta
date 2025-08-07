<script context="module" lang="ts">
    import type { RuntimeNodeDefinition } from '../stores/RuntimeNodeRegistry';
    
    export const runtimeNodeDef: RuntimeNodeDefinition = {
        ntype: 'runtime/fallback',
        displayName: 'Unknown Node',
        fallback: true
    };
</script>

<script lang="ts">
    import type { DataNode, ViewNode } from '../../types/types';
    
    export let dataNode: DataNode;
    export let viewNode: ViewNode; // External reference
    export let width: number;
    export let height: number;
    
    // Extract some basic info for display
    $: displayName = dataNode.attributes?.name || dataNode.path || dataNode.ntype;
    $: nodeType = dataNode.ntype;
</script>

<div 
    class="unknown-node"
    style="width: {width}px; height: {height}px;"
>
    <div class="content">
        <div class="icon">?</div>
        <div class="info">
            <div class="name">{displayName}</div>
            <div class="type">Type: {nodeType}</div>
        </div>
    </div>
</div>

<style>
    .unknown-node {
        background: #f3f4f6;
        border: 2px dashed #9ca3af;
        border-radius: 8px;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 12px;
        box-sizing: border-box;
    }
    
    .content {
        display: flex;
        flex-direction: column;
        align-items: center;
        text-align: center;
        gap: 8px;
        color: #6b7280;
    }
    
    .icon {
        font-size: 24px;
        font-weight: bold;
        color: #9ca3af;
    }
    
    .info {
        font-size: 12px;
        line-height: 1.3;
    }
    
    .name {
        font-weight: 600;
        color: #374151;
        word-break: break-word;
    }
    
    .type {
        color: #9ca3af;
        font-family: monospace;
    }
</style>
