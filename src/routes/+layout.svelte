<script lang="ts">
	import { onMount } from 'svelte';
	import '../app.css';
	import { initializeStores } from '$lib/karta/initialize'; // Import the initialization function

	let { children } = $props();

	onMount(() => {
		// Initialize the Karta stores
		initializeStores();

		const handleGlobalKeyDown = (event: KeyboardEvent) => {
			// Prevent default focus cycling for Tab key globally
			if (event.key === 'Tab') {
				event.preventDefault();
				console.log('[Layout] Global Tab key default prevented.');
			}
		};

		window.addEventListener('keydown', handleGlobalKeyDown);

		// Cleanup listener on component destroy
		return () => {
			window.removeEventListener('keydown', handleGlobalKeyDown);
		};
	});
</script>

{@render children()}
