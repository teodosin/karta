<script lang="ts">
    import { onMount } from 'svelte';
    
    export let id: string;
    export let x: number;
    export let y: number;
    export let content: string = 'Node';
    export let tool: string = 'move';
    
    let nodeElement: HTMLElement;
    let isDragging = false;
    let offsetX = 0;
    let offsetY = 0;
    
    onMount(() => {
      // Position the node
      updatePosition();
    });
    
    function updatePosition() {
      if (nodeElement) {
        nodeElement.style.transform = `translate(${x}px, ${y}px)`;
      }
    }
    
    function handleMouseDown(event: MouseEvent) {
      if (tool === 'move') {
        isDragging = true;
        
        // Calculate the offset from the mouse to the top-left corner of the node
        const rect = nodeElement.getBoundingClientRect();
        offsetX = event.clientX - rect.left;
        offsetY = event.clientY - rect.top;
        
        // Prevent event from bubbling to canvas
        event.stopPropagation();
      }
    }
    
    function handleMouseMove(event: MouseEvent) {
      if (isDragging && tool === 'move') {
        // Calculate new position
        const parentRect = nodeElement.parentElement?.getBoundingClientRect();
        if (parentRect) {
          x = event.clientX - parentRect.left - offsetX;
          y = event.clientY - parentRect.top - offsetY;
          updatePosition();
        }
      }
    }
    
    function handleMouseUp() {
      isDragging = false;
    }
  </script>
  
  <svelte:window 
    on:mousemove={handleMouseMove} 
    on:mouseup={handleMouseUp}
  />
  
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div 
    class="absolute w-50 min-h-[100px] bg-white rounded-lg shadow-md select-none z-10"
    bind:this={nodeElement}
    on:mousedown={handleMouseDown}
  >
    <div class="p-2 bg-gray-100 rounded-t-lg border-b border-gray-200 cursor-move">
      <div class="font-bold text-sm">{content}</div>
    </div>
    <div class="p-3">
      <slot>
        <p class="text-sm text-gray-700">Node content goes here</p>
      </slot>
    </div>
  </div>
  