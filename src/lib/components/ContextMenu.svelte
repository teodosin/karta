<script lang="ts">
	import type { ContextMenuContextType } from '$lib/karta/KartaStore'; // Import the type if needed for items later

	// Props
	export let position: { x: number; y: number } | null = null;
	// For now, items are just strings. Could be objects later { label: string, action: () => void }
	export let items: string[] = [];

	// TODO: Implement click outside logic later
</script>

{#if position}
	<div
		class="context-menu absolute bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded shadow-lg py-1 z-50"
		style="left: {position.x}px; top: {position.y}px;"
		role="menu"
		aria-orientation="vertical"
		aria-labelledby="options-menu"
	>
		{#if items.length > 0}
			<ul>
				{#each items as item}
					<li
						class="px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer"
						role="menuitem"
						on:click={() => console.log(`Context menu item clicked: ${item}`)}
					>
						{item}
					</li>
				{/each}
			</ul>
		{:else}
			<div class="px-4 py-2 text-sm text-gray-500 italic">(No actions available)</div>
		{/if}
	</div>
{/if}

<style>
	/* Add any specific styles if needed, Tailwind covers most */
	.context-menu {
		min-width: 150px; /* Ensure a minimum width */
	}
</style>