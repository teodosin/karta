<script lang="ts">
	import { onMount } from 'svelte';
	import type { ContextMenuContextType } from '$lib/karta/UIStateStore';
	import { closeContextMenu } from '$lib/karta/UIStateStore';

	// Props
	export let position: { x: number; y: number } | null = null; // Expects screen coordinates
	export let items: { label: string; action: () => void; disabled?: boolean }[] = [];

	let menuElement: HTMLDivElement | null = null;
	let adjustedPosition = { x: 0, y: 0 };

	function handleItemClick(item: { label: string; action: () => void; disabled?: boolean }) {
		if (!item.disabled) {
			item.action();
			closeContextMenu();
		}
	}

	function calculateBoundsAwarePosition() {
		if (!position || !menuElement) return;

		const rect = menuElement.getBoundingClientRect();
		const viewportWidth = window.innerWidth;
		const viewportHeight = window.innerHeight;
		
		let x = position.x;
		let y = position.y;

		// Minimum offset from click point to avoid covering it
		const minOffset = 10;

		// Check right boundary
		if (x + rect.width > viewportWidth) {
			x = position.x - rect.width - minOffset; // Position to the left of click
		} else {
			x = position.x + minOffset; // Default: slightly right of click
		}

		// Check bottom boundary
		if (y + rect.height > viewportHeight) {
			y = position.y - rect.height - minOffset; // Position above click
		} else {
			y = position.y + minOffset; // Default: slightly below click
		}

		// Ensure we don't go off the left edge
		x = Math.max(5, x);
		
		// Ensure we don't go off the top edge
		y = Math.max(5, y);

		adjustedPosition = { x, y };
	}

	// Recalculate position when menu mounts or position changes
	$: if (position && menuElement) {
		calculateBoundsAwarePosition();
	}

	onMount(() => {
		if (position && menuElement) {
			calculateBoundsAwarePosition();
		}
	});
</script>

{#if position}
	<div
		bind:this={menuElement}
		class="context-menu absolute bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded shadow-lg py-1 z-50"
		style="left: {adjustedPosition.x}px; top: {adjustedPosition.y}px;"
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