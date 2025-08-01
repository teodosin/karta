<script lang="ts">
	import { getAvailableNodeTypesForMenu } from '$lib/node_types/registry';
	import { createNodeFromMenu, closeCreateNodeMenu } from '$lib/karta/UIStateStore';
	import type { IconComponent } from '$lib/node_types/types';
	import { onMount, onDestroy, tick } from 'svelte';
	import { calculateBoundsAwarePosition } from '$lib/util/menuPositioning';

	// Define the structure returned by getAvailableNodeTypesForMenu
	interface AvailableNodeType {
		ntype: string;
		displayName: string;
		icon?: IconComponent;
	}

	// Position will be controlled by the parent/store
	export let position: { x: number; y: number } | null = null;

	let availableTypes: AvailableNodeType[] = [];
	let menuPanelRef: HTMLElement; // Reference to the menu panel div
	let adjustedPosition = { x: 0, y: 0 };

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

	async function updatePosition() {
		if (!position || !menuPanelRef) return;
		
		// Wait for DOM to be fully updated
		await tick();
		
		const rect = menuPanelRef.getBoundingClientRect();
		const viewportHeight = window.innerHeight;
		const viewportWidth = window.innerWidth;
		
		console.log('CreateNodeMenu Debug:', {
			position: position,
			menuRect: { width: rect.width, height: rect.height },
			viewport: { width: viewportWidth, height: viewportHeight },
			wouldExceedBottom: position.y + rect.height > viewportHeight,
			wouldExceedRight: position.x + rect.width > viewportWidth
		});
		
		adjustedPosition = calculateBoundsAwarePosition(position.x, position.y, menuPanelRef);
		
		console.log('CreateNodeMenu Adjusted:', adjustedPosition);
	}

	// Recalculate position when menu mounts or position changes
	$: if (position && menuPanelRef) {
		updatePosition();
	}

	onMount(() => {
		availableTypes = getAvailableNodeTypesForMenu();
		// Add global listener for clicks outside
		window.addEventListener('click', handleClickOutside, true); // Use capture phase
		
		// Calculate initial position
		if (position && menuPanelRef) {
			updatePosition();
		}
	});

	onDestroy(() => {
		// Remove global listener on cleanup
		window.removeEventListener('click', handleClickOutside, true);
	});
</script>

<svelte:window on:keydown={handleKeydown} />

{#if position}
	<!-- Menu Panel -->
	<div
		bind:this={menuPanelRef}
		class="absolute bg-gray-800 border border-gray-600 rounded-md shadow-lg p-2 z-50 min-w-[150px]"
		style="left: {adjustedPosition.x}px; top: {adjustedPosition.y}px;"
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
{/if}