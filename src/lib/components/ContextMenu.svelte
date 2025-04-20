<script lang="ts">
	import type { ContextMenuContextType } from '$lib/karta/UIStateStore'; // Import the type from UIStateStore
	import { closeContextMenu } from '$lib/karta/UIStateStore'; // Import close action from UIStateStore

	// Props
	export let position: { x: number; y: number } | null = null; // Expects screen coordinates
	export let items: { label: string; action: () => void; disabled?: boolean }[] = [];

	// TODO: Implement click outside logic later

	function handleItemClick(item: { label: string; action: () => void; disabled?: boolean }) {
		if (!item.disabled) {
			item.action();
			closeContextMenu(); // Close menu after action
		}
	}
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
						class="px-4 py-2 text-sm {item.disabled
							? 'text-gray-400 dark:text-gray-500 cursor-not-allowed'
							: 'text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer'}"
						role="menuitem"
						aria-disabled={item.disabled ? 'true' : 'false'}
						on:click={() => handleItemClick(item)}
						on:keydown={(e) => e.key === 'Enter' && handleItemClick(item)}
						tabindex={item.disabled ? -1 : 0}
					>
						{item.label}
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
	li:focus {
		outline: none; /* Remove default outline */
		background-color: #e5e7eb; /* Tailwind gray-200 */
	}
	:global(.dark) li:focus {
		background-color: #374151; /* Tailwind gray-700 */
	}
</style>