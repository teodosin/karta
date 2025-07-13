<script lang="ts">
	import { nodes } from '$lib/karta/NodeStore';
	import { currentContextId, switchContext, availableContextsMap } from '$lib/karta/ContextStore'; // Import availableContextsMap
	import { vaultName } from '$lib/karta/VaultStore';
	import { settings } from '$lib/karta/SettingsStore';
	import { get } from 'svelte/store'; // Import get
	// Remove fetchAvailableContextDetails import
	import { historyStack, futureStack, undoContextSwitch, redoContextSwitch } from '$lib/karta/HistoryStore';
	import type { NodeId, DataNode } from '$lib/types/types';
	import { onMount, onDestroy } from 'svelte'; // Import onMount and onDestroy

 let displayPath: string = '/'; // Default path
 let showContextList = false;
 let availableContexts: { id: NodeId; path: string }[] = []; // Removed 'name' from type
 // Remove isLoadingList
 let componentElement: HTMLElement; // Reference to the main component div for click outside

 // Reactively determine the path to display
 $: {
  const ctxId = $currentContextId;
  const contextNode: DataNode | undefined = $nodes.get(ctxId);
  if (ctxId === '00000000-0000-0000-0000-000000000000') {
 displayPath = '/root';
  }
  else if (contextNode && contextNode.path) {
   displayPath = '/' + contextNode.path.replace(/^vault/, $vaultName || 'vault');
  } else {
   displayPath = '/?';
  }
 }

 function toggleContextList() {
 	if (showContextList) {
 		showContextList = false;
 	} else {
 		// Populate directly from the store
 		const contextsMap = get(availableContextsMap);
 		availableContexts = Array.from(contextsMap.entries())
 			.map(([id, path]: [NodeId, string]) => ({ id, path })) // Add explicit types
 			.sort((a: { path: string }, b: { path: string }) => a.path.localeCompare(b.path)); // Add explicit types
 		showContextList = true;
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
	class="absolute bottom-2 left-2 px-3 py-1 text-sm rounded backdrop-blur-sm shadow bg-panel-bg flex items-center z-20"
	style="color: var(--color-text-color);"
>
	<!-- Undo Button -->
	<button
		type="button"
		class="text-xl hover:text-white hover:font-black rounded disabled:opacity-50 disabled:font-thin p-1 px-2"
		style="--panel-hl: {$settings.colorTheme['panel-hl']};"
		disabled={$historyStack.length === 0}
		on:click={undoContextSwitch}
		aria-label="Undo Context Switch"
	>
		←
	</button>
	<!-- Redo Button -->
	<button
		type="button"
		class="text-xl hover:text-white hover:font-black rounded disabled:opacity-50 disabled:font-thin p-1 px-2"
		style="--panel-hl: {$settings.colorTheme['panel-hl']};"
		disabled={$futureStack.length === 0}
		on:click={redoContextSwitch}
		aria-label="Redo Context Switch"
	>
		→
	</button>

	<!-- Container for Context Path Trigger and Dropdown -->
	<div class="relative ml-1">
		<!-- Context Path Trigger Button -->
		<button
			type="button"
			on:click={toggleContextList}
			class="flex items-center p-1 pl-2 pr-4 rounded"
			style="--panel-hl: {$settings.colorTheme['panel-hl']};"
			aria-haspopup="true"
			aria-expanded={showContextList}
		>
			<span class="bg-panel-bg p-1 px-2 rounded">Context</span>
			<span class="ml-2">{displayPath}</span>
		</button>

		<!-- Drop-up Context List (Now a sibling, positioned relative to the container) -->
		{#if showContextList}
			<div
				class="absolute bottom-full left-0 mb-1 w-64 max-h-96 overflow-y-auto rounded border border-gray-600 bg-gray-700 shadow-lg z-30"
				role="listbox"
			>
				<!-- Remove loading state -->
				{#if availableContexts.length === 0}
					<div class="px-2 py-1 text-gray-400">No contexts found.</div>
				{:else}
					{#each availableContexts as ctx (ctx.id)}
						<button
							role="option"
							class="block w-full text-left px-2 py-1 hover:bg-gray-600"
							on:click|stopPropagation={() => handleContextSelect(ctx.id)}
							title={ctx.path}
						>
							{ctx.path}
						</button>
					{/each}
				{/if}
			</div>
		{/if}
	</div>
</div>

<style>
	/* Custom Scrollbar for Context List Dropdown */
	div[role="listbox"]::-webkit-scrollbar {
		width: 3px; /* 3px width */
	}

	div[role="listbox"]::-webkit-scrollbar-track {
		background: transparent; /* Transparent track */
	}

	div[role="listbox"]::-webkit-scrollbar-thumb {
		/* Using Tailwind gray-500 with opacity for thumb */
		background-color: rgba(107, 114, 128, 0.5); /* gray-500 @ 50% */
		border-radius: 3px;
	}

	div[role="listbox"]::-webkit-scrollbar-thumb:hover {
		/* Using Tailwind gray-600 with opacity for hover */
		background-color: rgba(75, 85, 99, 0.7); /* gray-600 @ 70% */
	}

	/* Dark mode scrollbar thumb */
	:global(.dark) div[role="listbox"]::-webkit-scrollbar-thumb {
		/* Using Tailwind gray-400 with opacity for dark mode thumb */
		background-color: rgba(156, 163, 175, 0.5); /* gray-400 @ 50% */
	}

	:global(.dark) div[role="listbox"]::-webkit-scrollbar-thumb:hover {
		/* Using Tailwind gray-500 with opacity for dark mode hover */
		background-color: rgba(107, 114, 128, 0.7); /* gray-500 @ 70% */
	}
</style>
