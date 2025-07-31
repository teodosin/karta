<script lang="ts">
	import { Eye } from 'lucide-svelte';
	import { settings } from '$lib/karta/SettingsStore';
	import { isFilterMenuOpen, openFilterMenu, closeFilterMenu } from '$lib/karta/UIStateStore';
	import { onMount } from 'svelte';

	onMount(() => {
		console.log('FilterMenu button mounted!');
	});

	function toggleMenu(event: MouseEvent) {
		if ($isFilterMenuOpen) {
			closeFilterMenu();
		} else {
			const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
			const screenX = rect.right + 8; // 8px spacing to the right
			
			// Center the menu vertically on the screen
			const menuHeight = 300; // Approximate height of the filter menu
			const screenY = (window.innerHeight - menuHeight) / 2;
			
			openFilterMenu(screenX, screenY);
		}
	}
</script>

<div class="relative">
	<!-- Filter Button -->
	<div class="relative group">
		<button
			type="button"
			class="toolbar-button p-2 rounded transition-colors focus:outline-none focus:ring-2 hover-bg-panel-hl"
			style="{$isFilterMenuOpen ? `background-color: ${$settings.colorTheme['panel-hl']};` : ''}"
			on:click={toggleMenu}
			aria-label="Edge Filters"
			aria-expanded={$isFilterMenuOpen}
		>
			<Eye class="transition-all" strokeWidth={$isFilterMenuOpen ? 2.5 : 1.5} size={20} />
			<!-- Tooltip shown on hover -->
			<span
				class="absolute left-full top-1/2 transform -translate-y-1/2 ml-3 px-2 py-1 text-xs rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none bg-gray-900 text-white"
			>
				Edge Filters
			</span>
		</button>
	</div>
</div>

<style>
	.hover-bg-panel-hl:hover {
		background-color: var(--panel-hl);
	}
</style>
