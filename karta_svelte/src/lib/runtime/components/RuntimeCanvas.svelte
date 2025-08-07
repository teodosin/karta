<script lang="ts">
    import RuntimeViewport from './RuntimeViewport.svelte';
    import UnifiedNodeWrapper from './UnifiedNodeWrapper.svelte';
    import { runtimeContextStore } from '../stores/RuntimeContextStore';
    import { runtimeNodeStore } from '../stores/RuntimeNodeStore';
    import { runtimeViewportStore } from '../stores/RuntimeViewportStore';
    import { onMount } from 'svelte';
    
    export let contextId: string | undefined = undefined;
    export let canvasWidth: number = 800;
    export let canvasHeight: number = 600;
    
    let mounted = false;
    
    // Get the current context and its ViewNodes
    $: currentContext = contextId ? runtimeContextStore.getContext(contextId) : undefined;
    $: currentViewNodes = currentContext ? runtimeContextStore.getViewNodesInContext(contextId!) : new Map();
    $: viewNodesArray = Array.from(currentViewNodes.values());
    
    onMount(() => {
        mounted = true;
        
        // Center the viewport on load if we have nodes
        if (viewNodesArray.length > 0) {
            setTimeout(() => {
                frameAllNodes();
            }, 100);
        }
    });
    
    // Frame all nodes in the current context
    function frameAllNodes() {
        if (viewNodesArray.length === 0) return;
        
        let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
        
        viewNodesArray.forEach((viewNode) => {
            const state = viewNode.state.current;
            const nodeLeft = state.x - (state.width / 2) * state.scale;
            const nodeRight = state.x + (state.width / 2) * state.scale;
            const nodeTop = state.y - (state.height / 2) * state.scale;
            const nodeBottom = state.y + (state.height / 2) * state.scale;
            
            minX = Math.min(minX, nodeLeft);
            minY = Math.min(minY, nodeTop);
            maxX = Math.max(maxX, nodeRight);
            maxY = Math.max(maxY, nodeBottom);
        });
        
        const boundsWidth = maxX - minX;
        const boundsHeight = maxY - minY;
        const boundsCenterX = minX + boundsWidth / 2;
        const boundsCenterY = minY + boundsHeight / 2;
        
        if (boundsWidth > 0 && boundsHeight > 0) {
            // Calculate appropriate scale to fit all nodes with padding
            const padding = 50;
            const viewportWidth = 800; // Could get from store
            const viewportHeight = 600;
            
            const scaleX = (viewportWidth - padding * 2) / boundsWidth;
            const scaleY = (viewportHeight - padding * 2) / boundsHeight;
            const targetScale = Math.min(scaleX, scaleY, 1); // Don't zoom in beyond 100%
            
            // Center on the bounds
            const targetPosX = (viewportWidth / 2) - (boundsCenterX * targetScale);
            const targetPosY = (viewportHeight / 2) - (boundsCenterY * targetScale);
            
            runtimeViewportStore.setTransform({
                scale: targetScale,
                posX: targetPosX,
                posY: targetPosY
            });
        } else {
            // Single node or no bounds, just center on first node
            const firstNode = viewNodesArray[0];
            if (firstNode) {
                const state = firstNode.state.current;
                runtimeViewportStore.centerViewOnCanvasPoint(state.x, state.y);
            }
        }
    }
    
    // Debug info
    $: nodeCount = runtimeNodeStore.getNodeCount();
    $: contextCount = runtimeContextStore.getContextCount();
</script>

<div class="runtime-container">
    <div class="runtime-header">
        <h3>Karta Runtime</h3>
        <div class="runtime-stats">
            <span>Nodes: {nodeCount}</span>
            <span>Contexts: {contextCount}</span>
            {#if currentContext}
                <span>Context: {contextId}</span>
                <span>View Nodes: {viewNodesArray.length}</span>
            {/if}
        </div>
        {#if mounted && viewNodesArray.length > 0}
            <button on:click={frameAllNodes} class="frame-button">
                Frame All
            </button>
        {/if}
    </div>
    
    <div class="runtime-viewport-container">
        {#if currentContext}
            <RuntimeViewport {canvasWidth} {canvasHeight}>
                {#each viewNodesArray as viewNode (viewNode.id)}
                    {@const dataNode = runtimeNodeStore.getNode(viewNode.nodeId)}
                    {#if dataNode}
                        <UnifiedNodeWrapper 
                            {viewNode} 
                            {dataNode}
                            width={viewNode.state.current.width}
                            height={viewNode.state.current.height}
                        />
                    {/if}
                {/each}
            </RuntimeViewport>
        {:else}
            <div class="runtime-empty">
                <p>No context selected or context not found.</p>
                <p>Available contexts: {Array.from(runtimeContextStore.getAllContexts().keys()).join(', ')}</p>
            </div>
        {/if}
    </div>
</div>

<style>
    .runtime-container {
        display: flex;
        flex-direction: column;
        height: 100%;
        width: 100%;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    }
    
    .runtime-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 12px 16px;
        background: #f8f9fa;
        border-bottom: 1px solid #e9ecef;
        flex-shrink: 0;
    }
    
    .runtime-header h3 {
        margin: 0;
        font-size: 16px;
        font-weight: 600;
        color: #212529;
    }
    
    .runtime-stats {
        display: flex;
        gap: 16px;
        font-size: 12px;
        color: #6c757d;
    }
    
    .frame-button {
        padding: 6px 12px;
        font-size: 12px;
        background: #007bff;
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        transition: background-color 0.2s;
    }
    
    .frame-button:hover {
        background: #0056b3;
    }
    
    .runtime-viewport-container {
        flex: 1;
        overflow: hidden;
        position: relative;
    }
    
    .runtime-empty {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100%;
        color: #6c757d;
        text-align: center;
    }
    
    .runtime-empty p {
        margin: 8px 0;
        font-size: 14px;
    }
</style>
