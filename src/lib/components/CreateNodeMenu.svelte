<script lang="ts">
	import { getAvailableNodeTypesForMenu } from '$lib/node_types/registry';
	import { createNodeFromMenu, closeCreateNodeMenu } from '$lib/karta/UIStateStore';
	import type { IconComponent } from '$lib/node_types/types';
	import { onMount, onDestroy } from 'svelte';

	// Define the structure returned by getAvailableNodeTypesForMenu
	interface AvailableNodeType {
		ntype: string;
		displayName: string;
		icon?: IconComponent;
	}

	// Position will be controlled by the parent/store
	export let x: number;
	export let y: number;

	let availableTypes: AvailableNodeType[] = [];
	let menuPanelRef: HTMLElement; // Reference to the menu panel div

	function handleSelectType(ntype: string) {
		createNodeFromMenu(ntype); // This action will use the stored position
		// createNodeFromMenu calls closeCreateNodeMenu internally
	}

	// Close menu if clicked outside the menu panel
	function handleClickOutside(event: MouseEvent) {
		// Check if the menu panel exists and the click target is not inside it
		if (menuPanelRef && !menuPanelRef.contains(event.target as Node)) {
			closeCreateNodeMenu();
		}
	}

	// Handle Escape key to close
	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			closeCreateNodeMenu();
		}
	}

	onMount(() => {
		availableTypes = getAvailableNodeTypesForMenu();
		// Add global listener for clicks outside
		window.addEventListener('click', handleClickOutside, true); // Use capture phase
	});

	onDestroy(() => {
		// Remove global listener on cleanup
		window.removeEventListener('click', handleClickOutside, true);
	});
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- Menu Panel -->
<div
	bind:this={menuPanelRef}
	class="absolute bg-gray-800 border border-gray-600 rounded-md shadow-lg p-2 z-50 min-w-[150px]"
	style:left="{x}px"
	style:top="{y}px"
	role="menu"
>
	<ul class="space-y-1">
		{#each availableTypes as typeDef (typeDef.ntype)}
			<li>
				<button
					type="button"
					class="w-full text-left px-3 py-1.5 text-sm text-gray-200 hover:bg-gray-700 rounded flex items-center space-x-2"
					on:click={() => handleSelectType(typeDef.ntype)}
					role="menuitem"
				>
					{#if typeDef.icon}
						<svelte:component this={typeDef.icon} class="w-4 h-4 text-gray-400" />
					{/if}
					<span>{typeDef.displayName}</span>
				</button>
			</li>
		{/each}
	</ul>
</div>