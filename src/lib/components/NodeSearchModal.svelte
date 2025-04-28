<script lang="ts">
	import { onMount, onDestroy } from 'svelte'; // Corrected import
	import { get } from 'svelte/store'; // Corrected import
	import {
		isNodeSearchOpen,
		nodeSearchPosition,
		closeNodeSearch
	} from '$lib/karta/UIStateStore';
	import type { NodeId } from '$lib/types/types'; // Import NodeId type
	import { searchNodes, addExistingNodeToCurrentContext } from '$lib/karta/NodeStore'; // Import store actions
	import { fade } from 'svelte/transition';

	// Define the type for search results based on the optimized searchNodes return type
	type SearchResult = { id: NodeId, path: string, name: string };

	let searchInput: HTMLInputElement;
	let searchQuery = '';
	let searchResults: SearchResult[] = []; // Use the defined type
	let selectedIndex = -1; // For keyboard navigation

	// --- Positioning ---
	let modalElement: HTMLDivElement;
	let positionStyle = '';
	$: if ($nodeSearchPosition && modalElement) {
		// Position slightly offset from the click position
		const x = $nodeSearchPosition.screenX + 10;
		const y = $nodeSearchPosition.screenY + 10;
		// TODO: Add logic to prevent going off-screen
		positionStyle = `left: ${x}px; top: ${y}px;`;
	}

	// --- Search Logic ---
	async function performSearch() { // Make async
		console.log('Searching for:', searchQuery);
		searchResults = await searchNodes(searchQuery); // Call actual search function
		selectedIndex = -1; // Reset selection when results change
	}

	// Debounce search
	let debounceTimer: number;
	$: {
		clearTimeout(debounceTimer);
		// Trigger search only if query has content.
		if (searchQuery.trim().length > 0) {
			debounceTimer = window.setTimeout(performSearch, 300);
		} else {
			searchResults = []; // Clear results if query is empty
			selectedIndex = -1;
		}
	}

	// --- Event Handlers ---
	async function handleResultClick(result: SearchResult) { // Use SearchResult type, make async
		console.log('Selected result:', result);
		const position = get(nodeSearchPosition); // Get position from store
		if (position) {
			// Call action with only the ID and canvas coordinates
			await addExistingNodeToCurrentContext(result.id, { x: position.canvasX, y: position.canvasY });
		} else {
			console.error("Cannot add node: Search position is null.");
		}
		closeNodeSearch(); // Close modal after action
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			closeNodeSearch();
		} else if (event.key === 'ArrowDown') {
			event.preventDefault();
			selectedIndex = Math.min(selectedIndex + 1, searchResults.length - 1);
		} else if (event.key === 'ArrowUp') {
			event.preventDefault();
			selectedIndex = Math.max(selectedIndex - 1, 0); // Don't go below 0
		} else if (event.key === 'Enter') {
			event.preventDefault();
			if (selectedIndex >= 0 && selectedIndex < searchResults.length) {
				handleResultClick(searchResults[selectedIndex]);
			} else if (searchResults.length === 1) {
				// If only one result, Enter selects it even if not highlighted
				handleResultClick(searchResults[0]);
			}
		}
	}

	function handleClickOutside(event: MouseEvent) {
		// Use pointerdown to catch clicks before they might trigger other actions
		if ($isNodeSearchOpen && modalElement && !modalElement.contains(event.target as Node)) {
			closeNodeSearch();
		}
	}

	onMount(() => {
		// Focus the input when the modal opens
		searchInput?.focus();
		// Use pointerdown listener for click outside
		window.addEventListener('pointerdown', handleClickOutside, true); // Use capture phase
	});

	onDestroy(() => {
		window.removeEventListener('pointerdown', handleClickOutside, true);
		clearTimeout(debounceTimer);
	});
</script>

{#if $isNodeSearchOpen}
	<div
		bind:this={modalElement}
		class="absolute z-50 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded shadow-lg p-3 flex flex-col gap-2 min-w-[250px]"
		style={positionStyle}
		role="dialog"
		aria-modal="true"
		aria-labelledby="search-node-label"
		transition:fade={{ duration: 100 }}
		on:keydown={handleKeyDown}
		tabindex="-1"
	>
		<label id="search-node-label" for="search-node-input" class="text-sm font-medium text-gray-700 dark:text-gray-300">
			Search Node by Path/Name:
		</label>
		<input
			bind:this={searchInput}
			bind:value={searchQuery}
			id="search-node-input"
			type="text"
			placeholder="Enter node path or name..."
			class="w-full px-2 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-1 focus:ring-blue-500"
		/>

		{#if searchResults.length > 0}
			<ul class="mt-2 max-h-60 overflow-y-auto border border-gray-200 dark:border-gray-700 rounded" role="listbox">
				{#each searchResults as result, index (result.id)} <!-- Add key for better performance -->
					<!-- svelte-ignore a11y_click_events_have_key_events -->
					<li
						class="px-3 py-1.5 text-sm cursor-pointer truncate {selectedIndex === index
							? 'bg-blue-100 dark:bg-blue-900 text-blue-900 dark:text-blue-100'
							: 'text-gray-800 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700'}"
						class:selected={selectedIndex === index}
						on:click={() => handleResultClick(result)}
						on:mouseenter={() => (selectedIndex = index)}
						role="option"
						aria-selected={selectedIndex === index}
						id="search-result-{result.id}"
					>
						{result.name || result.path || result.id} <!-- Display name first, then path -->
						{#if result.name && result.path && result.name !== result.path}
							<span class="text-xs text-gray-500 dark:text-gray-400 ml-2">({result.path})</span> <!-- Show path if different from name -->
						{/if}
					</li>
				{/each}
			</ul>
		{:else if searchQuery.trim().length > 0} <!-- Check trimmed query -->
			<div class="mt-2 px-3 py-1.5 text-sm text-gray-500 italic">No matching nodes found.</div>
		{/if}
	</div>
{/if}

<style>
	/* Add any specific styles if needed */
	li.selected {
		/* Ensure selected style overrides hover */
		background-color: #dbeafe; /* Tailwind blue-100 */
		color: #1e40af; /* Tailwind blue-800 */
	}
	:global(.dark) li.selected {
		background-color: #1e3a8a; /* Tailwind blue-900 */
		color: #dbeafe; /* Tailwind blue-100 */
	}
</style>