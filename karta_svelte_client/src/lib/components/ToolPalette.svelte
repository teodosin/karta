<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    
    export let currentTool: string = 'move';
    
    const dispatch = createEventDispatcher();
    
    function selectTool(tool: string) {
      dispatch('toolChange', tool);
    }
  </script>
  
  <div class="absolute top-5 left-5 bg-white rounded-lg shadow-md p-2.5 flex flex-col gap-2 z-10">
    <!-- svelte-ignore a11y_consider_explicit_label -->
    <button 
      class="w-10 h-10 rounded flex items-center justify-center transition-colors {currentTool === 'move' ? 'bg-gray-200 text-black' : 'text-gray-700 hover:bg-gray-100'}" 
      on:click={() => selectTool('move')}
      title="Move Tool (M)"
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M18 11.5V9a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v1.4"></path>
        <path d="M14 10V8a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v2"></path>
        <path d="M10 10V6a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v10.5"></path>
        <path d="M6 16v2a2 2 0 0 0 2 2v0a2 2 0 0 0 2-2v-2"></path>
        <path d="M10 14v2a2 2 0 0 0 2 2v0a2 2 0 0 0 2-2v-1.5"></path>
        <path d="M14 14v1a2 2 0 0 0 2 2v0a2 2 0 0 0 2-2v-4"></path>
      </svg>
    </button>
  </div>
  