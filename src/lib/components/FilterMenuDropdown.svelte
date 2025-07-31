<script lang="ts">
	import { settings, updateSettings } from '$lib/karta/SettingsStore';
	import { closeFilterMenu } from '$lib/karta/UIStateStore';
	import type { EdgeVisibilityMode, EdgeFilterSettings } from '$lib/types/types';
	import { onMount, onDestroy } from 'svelte';

	export let x: number;
	export let y: number;

	let menuElement: HTMLDivElement;

	const visibilityOptions: { value: EdgeVisibilityMode; label: string }[] = [
		{ value: 'always', label: 'Show always' },
		{ value: 'all-selected', label: 'Show only on all selected' },
		{ value: 'single-selected', label: 'Show only on single selected' },
		{ value: 'never', label: 'Never' }
	];

	async function updateEdgeFilter(edgeType: 'containsEdges' | 'normalEdges', mode: EdgeVisibilityMode) {
		const newFilters: EdgeFilterSettings = {
			...$settings.edgeFilters,
			[edgeType]: mode
		};
		await updateSettings({ edgeFilters: newFilters });
	}

	// Close menu when clicking outside
	function handleClickOutside(event: MouseEvent) {
		if (menuElement && !menuElement.contains(event.target as Node)) {
			closeFilterMenu();
		}
	}

	// Handle escape key
	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			closeFilterMenu();
		}
	}

	onMount(() => {
		// Add global listeners for closing the menu
		window.addEventListener('click', handleClickOutside, true);
		window.addEventListener('keydown', handleKeydown);
	});

	onDestroy(() => {
		// Remove global listeners on cleanup
		window.removeEventListener('click', handleClickOutside, true);
		window.removeEventListener('keydown', handleKeydown);
	});
</script>

<!-- Filter Menu Dropdown -->
<div
	bind:this={menuElement}
	class="fixed w-64 bg-panel-bg border border-gray-600 rounded-md shadow-lg p-3"
	style="left: {x}px; top: {y}px; background-color: {$settings.colorTheme['panel-bg']}; z-index: 9999;"
>
	<div class="space-y-4">
		<h3 class="text-sm font-medium text-white mb-3">Edge Visibility</h3>
		
		<!-- Contains Edges Section -->
		<div class="space-y-2">
			<div class="text-xs font-medium text-gray-300 uppercase tracking-wide">
				Folder Connections
			</div>
			<div class="space-y-1">
				{#each visibilityOptions as option}
					<label class="flex items-center space-x-2 text-sm text-gray-100 hover:text-white cursor-pointer">
						<input
							type="radio"
							checked={$settings.edgeFilters.containsEdges === option.value}
							value={option.value}
							on:change={() => updateEdgeFilter('containsEdges', option.value)}
							class="w-3 h-3 text-blue-600 bg-gray-100 border-gray-300 focus:ring-blue-500 focus:ring-2"
						/>
						<span>{option.label}</span>
					</label>
				{/each}
			</div>
		</div>

		<!-- Divider -->
		<div class="h-px bg-gray-600/50 w-full"></div>

		<!-- Normal Edges Section -->
		<div class="space-y-2">
			<div class="text-xs font-medium text-gray-300 uppercase tracking-wide">
				Normal Edges
			</div>
			<div class="space-y-1">
				{#each visibilityOptions as option}
					<label class="flex items-center space-x-2 text-sm text-gray-100 hover:text-white cursor-pointer">
						<input
							type="radio"
							checked={$settings.edgeFilters.normalEdges === option.value}
							value={option.value}
							on:change={() => updateEdgeFilter('normalEdges', option.value)}
							class="w-3 h-3 text-blue-600 bg-gray-100 border-gray-300 focus:ring-blue-500 focus:ring-2"
						/>
						<span>{option.label}</span>
					</label>
				{/each}
			</div>
		</div>
	</div>
</div>
