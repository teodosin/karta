    <script lang="ts">
	import { Menu, X } from 'lucide-svelte';
	import { fly } from 'svelte/transition';

	let isOpen = false;

	// Placeholder functions - replace with actual logic later
	const handleImport = () => {
		console.log('Import Data clicked (placeholder)');
		alert('Import Data functionality not yet implemented.');
		isOpen = false; // Close menu after action
	};

	const handleExport = () => {
		console.log('Export Data clicked (placeholder)');
		alert('Export Data functionality not yet implemented.');
		isOpen = false; // Close menu after action
	};

	function toggleMenu() {
		isOpen = !isOpen;
	}
</script>

<div class="fixed top-2 left-2 z-50">
	<!-- Standard HTML button with Tailwind classes for styling -->
	<button
		on:click={toggleMenu}
		type="button"
		class="m-2 inline-flex items-center justify-center rounded-md p-1 text-foreground hover:bg-orange-800 hover:text-accent-foreground focus:outline-none focus:ring-2 focus:ring-inset focus:ring-primary"
		aria-label={isOpen ? 'Close Menu' : 'Open Menu'}
	>
		{#if isOpen}
			<X class="h-5 w-5" />
		{:else}
			<Menu class="h-5 w-5" />
		{/if}
	</button>

	{#if isOpen}
		<div
			class="absolute left-0 top-full m-2 w-56 rounded-md border border-orange-400 bg-wine p-2 text-white shadow-lg"
			transition:fly={{ y: -5, duration: 150 }}
		>
			<!-- Standard HTML buttons for menu items -->
			<button
				class="block w-full px-2 py-1.5 text-left text-sm hover:bg-orange-800 rounded-sm"
				on:click={handleImport}
			>
				Import Data...
			</button>
			<button
				class="block w-full px-2 py-1.5 text-left text-sm hover:bg-orange-800 rounded-sm"
				on:click={handleExport}
			>
				Export Data...
			</button>
			<div class="mt-2 border-t border-fuchsia-800 pt-2 text-sm text-fuchsia-100">
				Have any feedback or suggestions for future versions? Get in touch at 
					<a href="mailto:karta@teodosin.com" class="underline hover:text-white">
						karta@teodosin.com
					</a
				>
			</div>
		</div>
	{/if}
</div>