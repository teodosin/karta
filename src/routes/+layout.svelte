<script lang="ts">
	import { onMount } from 'svelte';
	import '../app.css';
	import { settings } from '$lib/karta/SettingsStore'; // Import the settings store
	import { initializeStores } from '$lib/karta/ContextStore'; // Import the main store initializer
	import { initializeVault } from '$lib/karta/VaultStore';
	import NodeSearchModal from '$lib/components/NodeSearchModal.svelte'; // Import the modal component

	let { children } = $props();

	$effect(() => {
		if ($settings.colorTheme) {
			const root = document.documentElement;

			for (const [key, value] of Object.entries($settings.colorTheme)) {
				root.style.setProperty(`--color-${key}`, value);
			}
		}
	});

	onMount(() => {
		// Define an async function for initialization
		const initializeApp = async () => {
			// Load settings from the server first
			await settings.loadSettings();

			// Initialize all other stores (nodes, contexts, etc.) AFTER settings are loaded
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
