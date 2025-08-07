<script lang="ts">
    import type { ViewNode } from '../../types/types';
    import { runtimeNodeStore } from '../stores/RuntimeNodeStore';
    import { runtimeNodeRegistry } from '../stores/RuntimeNodeRegistry';
    import { onMount } from 'svelte';
    
    export let viewNode: ViewNode;
    
    let registryReady = false;
    
    // Get the current state of the node
    $: nodeState = viewNode.state.current;
    
    // Get the actual DataNode from the store using the ViewNode's ID
    $: dataNode = runtimeNodeStore.getNode(viewNode.id);
    
    // Get the component for this node type (only when registry is ready)
    $: NodeComponent = (registryReady && dataNode) ? runtimeNodeRegistry.getComponent(dataNode.ntype) : null;
    
    // Calculate transform for positioning and scaling (center-based like editor)
    $: nodeTransform = `translate(${nodeState.x}px, ${nodeState.y}px) scale(${nodeState.scale}) rotate(${nodeState.rotation}deg) translateX(-50%) translateY(-50%)`;
    
    onMount(async () => {
        // Ensure registry is initialized
        await runtimeNodeRegistry.initialize();
        registryReady = true;
    });
</script>

{#if viewNode}
    <div 
        class="runtime-node-wrapper"
        style="transform: {nodeTransform}; width: {nodeState.width}px; height: {nodeState.height}px;"
        data-node-id={viewNode.id}
    >
        {#if dataNode && NodeComponent}
            <!-- Render the specific node type component -->
            <svelte:component 
                this={NodeComponent} 
                {dataNode} 
                {viewNode}
                width={nodeState.width}
                height={nodeState.height}
            />
        {:else if dataNode}
            <!-- Fallback for unknown node types -->
            <div class="unknown-node-fallback">
                <div class="fallback-content">
                    <div class="fallback-icon">?</div>
                    <div class="fallback-text">Unknown type: {dataNode.ntype}</div>
                </div>
            </div>
        {:else}
            <!-- Ghost node (DataNode not found) -->
            <div class="ghost-node">
                <div class="ghost-content">(Deleted)</div>
            </div>
        {/if}
    </div>
{/if}

<style>
    .runtime-node-wrapper {
        position: absolute;
        top: 0;
        left: 0;
        transform-origin: 50% 50%;
        pointer-events: auto;
    }
    
    .unknown-node-fallback {
        width: 100%;
        height: 100%;
        background: #f3f4f6;
        border: 2px dashed #9ca3af;
        border-radius: 8px;
        display: flex;
        align-items: center;
        justify-content: center;
        box-sizing: border-box;
    }
    
    .fallback-content {
        text-align: center;
        color: #6b7280;
    }
    
    .fallback-icon {
        font-size: 24px;
        font-weight: bold;
        margin-bottom: 4px;
    }
    
    .fallback-text {
        font-size: 12px;
        font-family: monospace;
    }
    
    .ghost-node {
        width: 100%;
        height: 100%;
        background: rgba(55, 65, 81, 0.2);
        border: 1px dashed #6b7280;
        border-radius: 8px;
        display: flex;
        align-items: center;
        justify-content: center;
        box-sizing: border-box;
        opacity: 0.4;
    }
    
    .ghost-content {
        color: #6b7280;
        font-style: italic;
        font-size: 14px;
    }
</style>
