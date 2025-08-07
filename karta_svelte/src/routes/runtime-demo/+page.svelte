<script lang="ts">
    import { onMount } from 'svelte';
    import { RuntimeCanvas, runtimeNodeStore, runtimeContextStore } from '../../lib/runtime';
    import type { DataNode, ContextBundle, StorableContext, StorableViewNode, StorableViewportSettings, TweenableNodeState } from '../../lib/types/types';
    
    let selectedContextId = '';
    
    onMount(() => {
        // Create some sample data for demonstration
        createSampleData();
    });
    
    function createSampleData() {
        // Create sample DataNodes
        const sampleNodes: DataNode[] = [
            {
                id: 'node-1',
                ntype: 'text',
                createdAt: Date.now(),
                modifiedAt: Date.now(),
                path: '/sample/text',
                attributes: { content: 'Hello World' },
                isSearchable: true
            },
            {
                id: 'node-2', 
                ntype: 'text',
                createdAt: Date.now(),
                modifiedAt: Date.now(),
                path: '/sample/text2',
                attributes: { content: 'This is another node' },
                isSearchable: true
            },
            {
                id: 'node-3',
                ntype: 'image',
                createdAt: Date.now(),
                modifiedAt: Date.now(),
                path: '/sample/image.jpg',
                attributes: { alt: 'Sample image' },
                isSearchable: true
            }
        ];
        
        // Load sample nodes
        runtimeNodeStore.loadNodes(sampleNodes);
        
        // Create sample context
        const storableViewNodes: [string, StorableViewNode][] = [
            ['node-1', {
                id: 'node-1',
                relX: 100,
                relY: 100,
                width: 200,
                height: 100,
                relScale: 1.0,
                rotation: 0,
                status: 'generated',
                attributes: {}
            }],
            ['node-2', {
                id: 'node-2',
                relX: 350,
                relY: 150,
                width: 250,
                height: 120,
                relScale: 1.0,
                rotation: 0,
                status: 'generated',
                attributes: {}
            }],
            ['node-3', {
                id: 'node-3',
                relX: 200,
                relY: 300,
                width: 180,
                height: 120,
                relScale: 1.0,
                rotation: 0,
                status: 'generated',
                attributes: {}
            }]
        ];
        
        const storableContext: StorableContext = {
            id: 'context-1',
            viewNodes: storableViewNodes,
            viewportSettings: {
                scale: 1.0,
                relPosX: 0,
                relPosY: 0
            }
        };
        
        const contextBundle: ContextBundle = {
            nodes: sampleNodes,
            edges: [],
            storableContext
        };
        
        runtimeContextStore.loadContextsFromBundle(contextBundle);
        selectedContextId = 'context-1';
    }
    
    const availableContexts = runtimeContextStore.getAllContexts();
</script>

<div class="runtime-demo">
    <header class="demo-header">
        <h1>Karta Runtime Demo</h1>
        <div class="context-selector">
            <label for="context-select">Context:</label>
            <select id="context-select" bind:value={selectedContextId}>
                <option value="">Select a context</option>
                {#each Array.from(availableContexts.keys()) as contextId}
                    <option value={contextId}>Context {contextId}</option>
                {/each}
            </select>
        </div>
    </header>
    
    <main class="demo-content">
        <RuntimeCanvas contextId={selectedContextId} canvasWidth={1000} canvasHeight={800} />
    </main>
</div>

<style>
    .runtime-demo {
        height: 100vh;
        display: flex;
        flex-direction: column;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    }
    
    .demo-header {
        background: #f8f9fa;
        padding: 16px 24px;
        border-bottom: 1px solid #e9ecef;
        display: flex;
        align-items: center;
        justify-content: space-between;
        flex-shrink: 0;
    }
    
    .demo-header h1 {
        margin: 0;
        font-size: 24px;
        font-weight: 600;
        color: #212529;
    }
    
    .context-selector {
        display: flex;
        align-items: center;
        gap: 12px;
    }
    
    .context-selector label {
        font-weight: 500;
        color: #495057;
    }
    
    .context-selector select {
        padding: 6px 12px;
        border: 1px solid #ced4da;
        border-radius: 4px;
        background: white;
        font-size: 14px;
    }
    
    .demo-content {
        flex: 1;
        overflow: hidden;
    }
</style>
