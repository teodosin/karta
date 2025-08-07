<script lang="ts">
    import { onMount } from 'svelte';
    import { runtimeViewportStore, runtimeViewTransform } from '../stores/RuntimeViewportStore';
    
    export let canvasWidth: number = 800;
    export let canvasHeight: number = 600;
    
    let viewport: HTMLDivElement;
    
    onMount(() => {
        if (viewport) {
            const rect = viewport.getBoundingClientRect();
            runtimeViewportStore.setDimensions(rect.width, rect.height);
            
            // Handle resize
            const resizeObserver = new ResizeObserver((entries) => {
                for (const entry of entries) {
                    const { width, height } = entry.contentRect;
                    runtimeViewportStore.setDimensions(width, height);
                }
            });
            resizeObserver.observe(viewport);
            
            return () => {
                resizeObserver.disconnect();
            };
        }
    });
    
    // Handle mouse wheel for zooming
    function handleWheel(event: WheelEvent) {
        event.preventDefault();
        const zoomFactor = event.deltaY > 0 ? 0.9 : 1.1;
        const rect = viewport.getBoundingClientRect();
        const pointX = event.clientX - rect.left;
        const pointY = event.clientY - rect.top;
        runtimeViewportStore.zoomAtPoint(zoomFactor, pointX, pointY);
    }
    
    // Handle mouse drag for panning
    let isDragging = false;
    let lastMouseX = 0;
    let lastMouseY = 0;
    
    function handleMouseDown(event: MouseEvent) {
        if (event.button === 0) { // Left mouse button
            isDragging = true;
            lastMouseX = event.clientX;
            lastMouseY = event.clientY;
            event.preventDefault();
        }
    }
    
    function handleMouseMove(event: MouseEvent) {
        if (isDragging) {
            const deltaX = event.clientX - lastMouseX;
            const deltaY = event.clientY - lastMouseY;
            
            const current = runtimeViewportStore.getCurrentTransform();
            runtimeViewportStore.setTransform({
                ...current,
                posX: current.posX + deltaX,
                posY: current.posY + deltaY
            }, false);
            
            lastMouseX = event.clientX;
            lastMouseY = event.clientY;
        }
    }
    
    function handleMouseUp() {
        isDragging = false;
    }
    
    // Calculate canvas transform style (reactive)
    $: canvasTransform = `translate(${runtimeViewTransform.current.posX || 0}px, ${runtimeViewTransform.current.posY || 0}px) scale(${runtimeViewTransform.current.scale || 1})`;
</script>

<!-- svelte-ignore a11y-no-noninteractive-tabindex -->
<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
<svelte:window on:mousemove={handleMouseMove} on:mouseup={handleMouseUp} />

<!-- Main viewport container that handles all interactions -->
<!-- svelte-ignore a11y-no-noninteractive-tabindex -->
<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
<div 
    bind:this={viewport}
    class="runtime-viewport"
    on:wheel|preventDefault={handleWheel}
    on:mousedown={handleMouseDown}
    role="application"
    tabindex="0"
    aria-label="Karta runtime viewport"
>
    <div 
        class="runtime-canvas"
        style="transform: {canvasTransform}; width: {canvasWidth}px; height: {canvasHeight}px;"
    >
        <slot />
    </div>
</div>

<style>
    .runtime-viewport {
        position: relative;
        width: 100%;
        height: 100%;
        overflow: hidden;
        cursor: grab;
        background: #f8f9fa;
        border: 1px solid #e9ecef;
    }
    
    .runtime-viewport:active {
        cursor: grabbing;
    }
    
    .runtime-canvas {
        position: absolute;
        top: 0;
        left: 0;
        transform-origin: 0 0;
        background: white;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    }
</style>
