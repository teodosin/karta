<script lang="ts">
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import '../app.css';
	import { settings } from '$lib/karta/SettingsStore';
	import { initializeStores } from '$lib/karta/ContextStore';
	import { initializeVault } from '$lib/karta/VaultStore';
	import NodeSearchModal from '$lib/components/NodeSearchModal.svelte';
    import { initializeTools } from '$lib/karta/ToolStore';

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
		const initializeApp = async () => {
			await settings.loadSettings();

			// Check if we're in Tauri mode
			let isTauriApp = false;
			if (browser && typeof window !== 'undefined' && typeof (window as any).isTauri === 'function') {
				isTauriApp = (window as any).isTauri();
			} else {
				isTauriApp = browser && '__TAURI__' in window;
			}

			// Only initialize server-dependent stores in web mode
			// In Tauri mode, ServerSetupModal will handle this after server is ready
			if (!isTauriApp) {
				await initializeVault();
				await initializeStores();
			}
			
			// Always initialize tools (not server-dependent)
			await initializeTools();
		};


		initializeApp();

		const handleGlobalKeyDown = (event: KeyboardEvent) => {
			if (event.key === 'Tab') {
				event.preventDefault();
			}
		};

		window.addEventListener('keydown', handleGlobalKeyDown);

		return () => {
			window.removeEventListener('keydown', handleGlobalKeyDown);
		};
	});

</script>

{@render children()}

<NodeSearchModal />
