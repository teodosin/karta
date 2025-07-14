<script lang="ts">
	import { onMount } from 'svelte';
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

			await initializeVault();
			await initializeTools();
			await initializeStores();
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
