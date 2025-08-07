<script context="module" lang="ts">
    import type { RuntimeNodeDefinition } from '../../stores/RuntimeNodeRegistry';
    
    export const runtimeNodeDef: RuntimeNodeDefinition = {
        ntype: 'core/text',
        displayName: 'Text'
    };
</script>

<script lang="ts">
    import type { DataNode, ViewNode } from '../../../types/types';
    
    export let dataNode: DataNode;
    export let viewNode: ViewNode; // External reference
    export let width: number;
    export let height: number;
    
    // Get text content from attributes
    $: textContent = dataNode.attributes?.type_text || dataNode.attributes?.content || 'Empty text';
    
    // Get styling attributes with fallbacks
    $: fillColor = dataNode.attributes?.viewtype_fillColor || viewNode.attributes?.viewtype_fillColor || '#FEF9C3';
    $: textColor = dataNode.attributes?.viewtype_textColor || viewNode.attributes?.viewtype_textColor || '#000000';
    $: fontSize = dataNode.attributes?.viewtype_fontSize || viewNode.attributes?.viewtype_fontSize || 16;
    $: font = dataNode.attributes?.viewtype_font || viewNode.attributes?.viewtype_font || 'Nunito';
</script>

<div 
    class="text-node"
    style="
        width: {width}px; 
        height: {height}px;
        background-color: {fillColor};
        color: {textColor};
        font-family: {font}, sans-serif;
        font-size: {fontSize}px;
    "
>
    <div class="content">
        {textContent}
    </div>
</div>

<style>
    .text-node {
        border: 2px solid #3b82f6;
        border-radius: 8px;
        padding: 12px;
        box-sizing: border-box;
        display: flex;
        align-items: center;
        justify-content: center;
        overflow: hidden;
        backdrop-filter: blur(8px);
    }
    
    .content {
        width: 100%;
        height: 100%;
        overflow: hidden;
        word-wrap: break-word;
        line-height: 1.4;
        display: flex;
        align-items: center;
        justify-content: center;
        text-align: center;
    }
</style>
