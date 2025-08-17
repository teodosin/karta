<script lang="ts">
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import '../app.css';
	import { settings } from '$lib/karta/SettingsStore';
	import { initializeStores } from '$lib/karta/ContextStore';
	import { initializeVault } from '$lib/karta/VaultStore';
	import { serverManager } from '$lib/tauri/server';
	import NodeSearchModal from '$lib/components/NodeSearchModal.svelte';
	import TutorialModal from '$lib/components/TutorialModal.svelte';
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
			// Check if we're in Tauri mode early
			let isTauriApp = false;
			if (browser && typeof window !== 'undefined' && typeof (window as any).isTauri === 'function') {
				isTauriApp = (window as any).isTauri();
			} else {
				isTauriApp = browser && '__TAURI__' in window;
			}

			if (!isTauriApp) {
				// Web mode: server is expected to be reachable; load settings and init stores now
				await settings.loadSettings();
				await initializeVault();
				await initializeStores();
			} else {
				// Tauri mode: the backend server may not be running yet.
				// Avoid early fetches that trigger CORS/network errors; load settings only if server is up.
				try {
					const up = await serverManager.checkServerStatus();
					if (up) {
						await settings.loadSettings();
					}
				} catch (e) {
					// Ignore; ServerSetupModal will handle once server starts
				}
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
<TutorialModal />
