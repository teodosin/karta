<script lang="ts">
	import {
		nodes,
		currentContextId,
		historyStack,
		futureStack,
		undoContextSwitch,
		redoContextSwitch,
		fetchAvailableContextDetails, // Import the new function
		switchContext // Import switchContext for list items
	} from '$lib/karta/KartaStore';
	import type { NodeId, DataNode } from '$lib/types/types';
	import { onMount, onDestroy } from 'svelte'; // Import onMount and onDestroy

 let displayPath: string = '/'; // Default path
 let showContextList = false;
 let availableContexts: { id: NodeId; name: string; path: string }[] = [];
 let isLoadingList = false;
 let componentElement: HTMLElement; // Reference to the main component div for click outside

 // Reactively determine the path to display
 $: {
  const ctxId = $currentContextId;
  const contextNode: DataNode | undefined = $nodes.get(ctxId);
  if (contextNode && contextNode.path) {
   displayPath = contextNode.path;
  } else {
   displayPath = '/?';
  }
 }

 async function toggleContextList() {
  if (showContextList) {
   showContextList = false;
  } else {
   isLoadingList = true;
   showContextList = true; // Show list immediately with loading state
   try {
    availableContexts = await fetchAvailableContextDetails();
   } catch (error) {
    console.error('Error fetching context list:', error);
    availableContexts = []; // Clear list on error
   } finally {
    isLoadingList = false;
   }
  }
 }

 function handleContextSelect(id: NodeId) {
  switchContext(id);
  showContextList = false; // Close list after selection
 }

 // Close dropdown if clicked outside the main component element
 function handleClickOutside(event: MouseEvent) {
 	if (showContextList && componentElement && !componentElement.contains(event.target as Node)) {
 		showContextList = false;
 	}
 }

 onMount(() => {
  window.addEventListener('click', handleClickOutside, true); // Use capture phase
  return () => {
   window.removeEventListener('click', handleClickOutside, true);
  };
 });

 // Handle Escape key to close the list
 function handleKeyDown(event: KeyboardEvent) {
     if (showContextList && event.key === 'Escape') {
         showContextList = false;
     }
 }
</script>

<svelte:window on:keydown={handleKeyDown} />

<!-- Bind the component's root element -->
<div
	bind:this={componentElement}
	class="absolute bottom-2 left-2 p-1 text-sm rounded shadow bg-gray-800 text-gray-300 flex items-center z-20"
>
	<!-- Undo Button -->
	<button
		type="button"
		class="text-xl hover:text-white hover:font-black hover:bg-gray-600 rounded disabled:opacity-50 disabled:font-thin p-1 px-2"
		disabled={$historyStack.length === 0}
		on:click={undoContextSwitch}
		aria-label="Undo Context Switch"
	>
		←
	</button>
	<!-- Redo Button -->
	<button
		type="button"
		class="text-xl hover:text-white hover:font-black hover:bg-gray-600 rounded disabled:opacity-50 disabled:font-thin p-1 px-2"
		disabled={$futureStack.length === 0}
		on:click={redoContextSwitch}
		aria-label="Redo Context Switch"
	>
		→
	</button>

	<!-- Context Path Trigger Button -->
	<button
		type="button"
		on:click={toggleContextList}
		class="flex items-center hover:bg-gray-700 p-1 rounded ml-1 relative"
		aria-haspopup="true"
		aria-expanded={showContextList}
	>
		<span class="bg-pink-900 p-1 px-2 rounded">Context</span>
		<span class="ml-2">{displayPath}</span>

		<!-- Drop-up Context List -->
		{#if showContextList}
			<div
				class="absolute bottom-full left-0 mb-1 w-64 max-h-48 overflow-y-auto rounded border border-gray-600 bg-gray-700 shadow-lg z-30"
				role="listbox"
				> <!-- Removed bind:this -->
				{#if isLoadingList}
					<div class="px-2 py-1 text-gray-400">Loading...</div>
				{:else if availableContexts.length === 0}
					<div class="px-2 py-1 text-gray-400">No contexts found.</div>
				{:else}
					{#each availableContexts as ctx (ctx.id)}
						<button
							role="option"
							class="block w-full text-left px-2 py-1 hover:bg-gray-600"
							on:click|stopPropagation={() => handleContextSelect(ctx.id)}
							title={ctx.path}
						>
							{ctx.name} <span class="text-xs text-gray-400">({ctx.path})</span>
						</button>
					{/each}
				{/if}
			</div>
		{/if}
	</button>
</div>
