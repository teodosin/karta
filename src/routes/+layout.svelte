<script lang="ts">
	import { onMount } from 'svelte';
	import '../app.css';
	import { settings } from '$lib/karta/SettingsStore'; // Import the settings store
	import { initializeStores } from '$lib/karta/ContextStore'; // Import the main store initializer
	import { initializeVault } from '$lib/karta/VaultStore';
	import NodeSearchModal from '$lib/components/NodeSearchModal.svelte'; // Import the modal component

	let { children } = $props();

	onMount(() => { // Remove async here
		// Define an async function for initialization
		const initializeApp = async () => {
			// Load settings from localStorage first
			settings.loadSettings();

			// Initialize all other stores (nodes, contexts, etc.) AFTER settings are loaded
			// This ensures ContextStore reads the correct setting for loading the last context
			await initializeStores();
			await initializeVault();
		};

		// Call the async initialization function
		initializeApp();

		const handleGlobalKeyDown = (event: KeyboardEvent) => {
			// Prevent default focus cycling for Tab key globally
			if (event.key === 'Tab') {
				event.preventDefault();
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

<NodeSearchModal />
