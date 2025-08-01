<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { get } from 'svelte/store';
	import { browser } from '$app/environment';
	import { localAdapter } from '$lib/util/LocalAdapter';
	import {
		isNodeSearchOpen,
		nodeSearchPosition,
		closeNodeSearch
	} from '$lib/karta/UIStateStore';
	import { addExistingNodeToCurrentContext } from '$lib/karta/NodeStore';
	import { fade, scale } from 'svelte/transition';

	let searchInput: HTMLInputElement;
	let searchQuery = '';
	let allNodePaths: string[] = [];
	let filteredNodePaths: string[] = [];
	let selectedIndex = -1;

	// --- Destination Marker Positioning ---
	let destinationMarkerStyle = '';
	$: if ($nodeSearchPosition) {
		// Use screen coordinates for the destination marker
		const x = $nodeSearchPosition.screenX;
		const y = $nodeSearchPosition.screenY;
		destinationMarkerStyle = `left: ${x}px; top: ${y}px;`;
	}

	// --- Search Logic (Local Filtering) ---
	let modalElement: HTMLDivElement;

	// Reactive block to filter paths based on search query
	$: {
		if (!allNodePaths) {
			filteredNodePaths = [];
		} else if (searchQuery.trim().length > 0) {
			const lowerCaseQuery = searchQuery.toLowerCase();
			filteredNodePaths = allNodePaths.filter(path =>
				path.toLowerCase().includes(lowerCaseQuery)
			);
		} else {
			filteredNodePaths = [...allNodePaths];
		}
		if (selectedIndex >= filteredNodePaths.length) {
			selectedIndex = -1;
		} else if (filteredNodePaths.length > 0 && selectedIndex === -1) {
			// Optionally select the first item automatically
		} else if (filteredNodePaths.length === 0) {
			selectedIndex = -1;
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
		// The backdrop div will handle closing, so we don't need this for the modal itself
		// Keep this for any future use cases where clicks need to be handled differently
		if ($isNodeSearchOpen && modalElement && !modalElement.contains(event.target as Node)) {
			// Don't close here since backdrop handles it
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
	<!-- Transparent backdrop for click-to-close -->
	<div 
		class="fixed inset-0 z-40"
		transition:fade={{ duration: 200 }}
		on:click={closeNodeSearch}
	></div>

	<!-- Pulsating destination marker -->
	{#if $nodeSearchPosition}
		<div
			class="fixed z-45 pointer-events-none"
			style={destinationMarkerStyle}
		>
			<!-- Central white dot that fades in/out -->
			<div class="absolute w-1.5 h-1.5 bg-white rounded-full shadow-lg transform -translate-x-1/2 -translate-y-1/2" transition:fade={{ duration: 300 }}></div>
			
			<!-- First pulsating expanding ring -->
			<div class="absolute w-0 h-0 border-2 border-white rounded-full transform -translate-x-1/2 -translate-y-1/2 animate-ring-pulse"></div>
			
			<!-- Second pulsating expanding ring (delayed) -->
			<div class="absolute w-0 h-0 border-2 border-white rounded-full transform -translate-x-1/2 -translate-y-1/2 animate-ring-pulse-delayed"></div>
		</div>
	{/if}

	<!-- Large centered modal -->
	<div
		bind:this={modalElement}
		class="fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 z-50 
		       border border-gray-700 rounded-lg shadow-2xl w-[500px] max-w-[90vw] h-[400px] flex flex-col"
		style="background-color: color-mix(in srgb, var(--color-panel-bg) 90%, transparent);"
		role="dialog"
		aria-modal="true"
		aria-labelledby="search-node-label"
		transition:scale={{ duration: 200, start: 0.95 }}
		on:keydown={handleKeyDown}
		tabindex="-1"
	>
		<!-- Modal header -->
		<div class="p-3 border-b border-gray-700">
			<label id="search-node-label" for="search-node-input" class="text-sm font-medium" style="color: var(--color-text-color);">
				Search Nodes
			</label>
			
			<!-- Search input -->
			<input
				bind:this={searchInput}
				bind:value={searchQuery}
				id="search-node-input"
				type="text"
				placeholder="Type to search..."
				class="w-full mt-2 px-3 py-2 text-sm border border-gray-700 rounded 
				       text-white placeholder-gray-400
				       focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-transparent"
				style="background-color: var(--color-viewport-bg);"
			/>
		</div>

		<!-- Results container (fixed height with scroll) -->
		<div class="flex-1 overflow-hidden">
			{#if filteredNodePaths.length > 0}
				<ul class="h-full overflow-y-auto" role="listbox">
					{#each filteredNodePaths as path, index (path)}
						<!-- svelte-ignore a11y_click_events_have_key_events -->
						<li
							class="px-3 py-2 text-xs cursor-pointer border-b border-gray-700 last:border-b-0
							       {selectedIndex === index
								? 'hover-bg-panel-hl'
								: 'hover:opacity-80'}"
							style="color: var(--color-text-color);"
							on:click={() => handleResultClick(path)}
							on:mouseenter={() => (selectedIndex = index)}
							role="option"
							aria-selected={selectedIndex === index}
							id="search-result-{index}"
						>
							<div class="font-mono truncate">
								{path}
							</div>
						</li>
					{/each}
				</ul>
			{:else if searchQuery.trim().length > 0 && allNodePaths.length > 0}
				<div class="p-6 text-center">
					<div class="text-gray-400">
						<div class="text-2xl mb-2">🔍</div>
						<div class="text-sm font-medium mb-1">No matches</div>
						<div class="text-xs">Try a different search term</div>
					</div>
				</div>
			{:else if allNodePaths.length === 0 && searchQuery.trim().length === 0}
				<div class="p-6 text-center">
					<div class="text-gray-400">
						<div class="animate-spin text-lg mb-2">⟳</div>
						<div class="text-sm font-medium">Loading...</div>
					</div>
				</div>
			{:else}
				<div class="p-6 text-center">
					<div class="text-gray-400">
						<div class="text-sm font-medium mb-1">Start typing</div>
						<div class="text-xs">Search nodes by path</div>
					</div>
				</div>
			{/if}
		</div>

		<!-- Modal footer with shortcuts -->
		<div class="p-2 border-t border-gray-700" style="background-color: color-mix(in srgb, var(--color-panel-bg) 80%, black);">
			<div class="flex items-center justify-between text-xs text-gray-400">
				<div class="flex items-center gap-3">
					<span><kbd class="px-1 py-0.5 bg-gray-700 rounded text-xs">↑↓</kbd> Navigate</span>
					<span><kbd class="px-1 py-0.5 bg-gray-700 rounded text-xs">Enter</kbd> Add</span>
				</div>
				<span><kbd class="px-1 py-0.5 bg-gray-700 rounded text-xs">Esc</kbd> Close</span>
			</div>
		</div>
	</div>
{/if}

<style>
	/* Custom scrollbar for search results */
	ul::-webkit-scrollbar {
		width: 6px;
	}

	ul::-webkit-scrollbar-track {
		background: transparent;
	}

	ul::-webkit-scrollbar-thumb {
		background-color: rgba(156, 163, 175, 0.5);
		border-radius: 3px;
	}

	ul::-webkit-scrollbar-thumb:hover {
		background-color: rgba(107, 114, 128, 0.7);
	}

	:global(.dark) ul::-webkit-scrollbar-thumb {
		background-color: rgba(107, 114, 128, 0.5);
	}

	:global(.dark) ul::-webkit-scrollbar-thumb:hover {
		background-color: rgba(156, 163, 175, 0.7);
	}

	/* Keyboard shortcuts styling */
	kbd {
		font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
		font-size: 0.75rem;
		font-weight: 500;
	}

	/* Custom marker animations */
	@keyframes ring-pulse {
		0% {
			transform: translate(-50%, -50%) scale(0);
			opacity: 1;
		}
		100% {
			transform: translate(-50%, -50%) scale(4);
			opacity: 0;
		}
	}

	@keyframes ring-pulse-delayed {
		0% {
			transform: translate(-50%, -50%) scale(0);
			opacity: 0.8;
		}
		100% {
			transform: translate(-50%, -50%) scale(3.5);
			opacity: 0;
		}
	}

	.animate-ring-pulse {
		animation: ring-pulse 2s ease-out infinite;
	}

	.animate-ring-pulse-delayed {
		animation: ring-pulse-delayed 2s ease-out infinite 0.4s;
	}
</style>