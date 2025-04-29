<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { get } from 'svelte/store';
	import { browser } from '$app/environment'; // Import browser environment variable
	import { localAdapter } from '$lib/util/LocalAdapter'; // Import localAdapter
	import {
		isNodeSearchOpen,
		nodeSearchPosition,
		closeNodeSearch
	} from '$lib/karta/UIStateStore';
	// NodeId type might not be needed here anymore
	import { addExistingNodeToCurrentContext } from '$lib/karta/NodeStore'; // Keep addExistingNodeToCurrentContext
	import { fade } from 'svelte/transition';

	// Removed SearchResult type

	let searchInput: HTMLInputElement;
	let searchQuery = '';
	let allNodePaths: string[] = []; // Store all fetched paths
	let filteredNodePaths: string[] = []; // Store filtered paths for display
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

	// --- Search Logic (Local Filtering) ---
	// No need for performSearch or debounceTimer

	// Reactive block to filter paths based on search query
	$: {
		if (!allNodePaths) {
			filteredNodePaths = []; // Handle case where paths haven't loaded yet
		} else if (searchQuery.trim().length > 0) {
			const lowerCaseQuery = searchQuery.toLowerCase();
			// Basic filter (TODO: Replace with fuzzy search library like fuse.js later)
			filteredNodePaths = allNodePaths.filter(path =>
				path.toLowerCase().includes(lowerCaseQuery)
			);
		} else {
			// Show all paths if query is empty (create new array reference)
			filteredNodePaths = [...allNodePaths];
		}
		if (selectedIndex >= filteredNodePaths.length) {
			selectedIndex = -1; // Reset selection if it becomes invalid after filtering
		} else if (filteredNodePaths.length > 0 && selectedIndex === -1) {
	            // Optionally select the first item automatically? Or leave as -1.
	            // selectedIndex = 0;
	       } else if (filteredNodePaths.length === 0) {
	           selectedIndex = -1; // Ensure reset if no results
	       }
	}

	// --- Event Handlers ---
	async function handleResultClick(path: string) { // Accept path string
		const position = get(nodeSearchPosition); // Get position from store
		if (position && path) {
			// Call action with the selected PATH and canvas coordinates
			await addExistingNodeToCurrentContext(path, { x: position.canvasX, y: position.canvasY });
		} else {
		}
		closeNodeSearch(); // Close modal after action
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			closeNodeSearch();
		} else if (event.key === 'ArrowDown') {
			event.preventDefault();
			selectedIndex = Math.min(selectedIndex + 1, filteredNodePaths.length - 1);
		} else if (event.key === 'ArrowUp') {
			event.preventDefault();
			selectedIndex = Math.max(selectedIndex - 1, 0); // Don't go below 0
		} else if (event.key === 'Enter') {
			event.preventDefault();
			if (selectedIndex >= 0 && selectedIndex < filteredNodePaths.length) {
				handleResultClick(filteredNodePaths[selectedIndex]); // Pass selected path
			} else if (filteredNodePaths.length === 1) {
				// If only one result, Enter selects it even if not highlighted
				handleResultClick(filteredNodePaths[0]); // Pass the single path
			}
		}
	}

	function handleClickOutside(event: MouseEvent) {
		// Use pointerdown to catch clicks before they might trigger other actions
		if ($isNodeSearchOpen && modalElement && !modalElement.contains(event.target as Node)) {
			closeNodeSearch();
		}
	}

	async function fetchNodePaths() {
		if (localAdapter) {
			try {
				allNodePaths = await localAdapter.getAllNodePaths();
				filteredNodePaths = [...allNodePaths]; // Initialize filtered list (create new array reference)
			} catch (error) {
				allNodePaths = [];
				filteredNodePaths = [];
			}
		} else {
		}
	}

	onMount(() => {
		// Use pointerdown listener for click outside, only in browser
		if (browser) {
			window.addEventListener('pointerdown', handleClickOutside, true); // Use capture phase
		}
	});

	// Reactive statement to fetch paths and focus input when modal opens
	$: if ($isNodeSearchOpen) {
		fetchNodePaths();
		// Focus the input when the modal opens
		searchInput?.focus();
	}

	onDestroy(() => {
		// Remove listener only in browser
		if (browser) {
			window.removeEventListener('pointerdown', handleClickOutside, true);
		}
		// clearTimeout(debounceTimer); // Removed leftover timer clear
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
			Search Node by Path:
		</label>
		<input
			bind:this={searchInput}
			bind:value={searchQuery}
			id="search-node-input"
			type="text"
			placeholder="Enter node path..."
			class="w-full px-2 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-1 focus:ring-blue-500"
		/>

		{#if filteredNodePaths.length > 0}
			<ul class="mt-2 max-h-60 overflow-y-auto border border-gray-200 dark:border-gray-700 rounded" role="listbox">
				{#each filteredNodePaths as path, index (path)} <!-- Iterate over paths, use path as key -->
					<!-- svelte-ignore a11y_click_events_have_key_events -->
					<li
						class="px-3 py-1.5 text-sm cursor-pointer truncate {selectedIndex === index
							? 'bg-blue-100 dark:bg-blue-900 text-blue-900 dark:text-blue-100'
							: 'text-gray-800 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700'}"
						class:selected={selectedIndex === index}
						on:click={() => handleResultClick(path)}
						on:mouseenter={() => (selectedIndex = index)}
						role="option"
						aria-selected={selectedIndex === index}
						id="search-result-{index}"
					>
						{path} <!-- Display the path -->
					</li>
				{/each}
			</ul>
		{:else if searchQuery.trim().length > 0 && allNodePaths.length > 0} <!-- Show 'not found' only if search attempted on existing paths -->
			<div class="mt-2 px-3 py-1.5 text-sm text-gray-500 italic">No matching nodes found.</div>
			     {:else if allNodePaths.length === 0 && searchQuery.trim().length === 0} <!-- Indicate loading or no nodes -->
			          <div class="mt-2 px-3 py-1.5 text-sm text-gray-500 italic">Loading node paths...</div>
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