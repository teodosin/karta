<script lang="ts">
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import Viewport from '$lib/components/Viewport.svelte';
	import Toolbar from '$lib/components/Toolbar.svelte';
	import ContextPathDisplay from '$lib/components/ContextPathDisplay.svelte';
	import KartaDebugOverlay from '$lib/components/KartaDebugOverlay.svelte';
	import PropertiesPanel from '$lib/components/PropertiesPanel.svelte';
	import AppMenu from '$lib/components/AppMenu.svelte';
	import Notification from '$lib/components/Notification.svelte';
	import ServerSetupModal from '$lib/components/ServerSetupModal.svelte';
	import { notifications } from '$lib/karta/NotificationStore';
	import ColorPickerPopup from '$lib/components/ColorPickerPopup.svelte';

	let isServerReady = false;
	let isTauriApp = false;

	onMount(async () => {
		// Check if we're running in a Tauri app using the official v2 method
		if (browser && typeof window !== 'undefined' && typeof (window as any).isTauri === 'function') {
			isTauriApp = (window as any).isTauri();
			console.log('Tauri detection via window.isTauri():', isTauriApp);
		} else {
			// Fallback: check for __TAURI__ global
			isTauriApp = browser && '__TAURI__' in window;
			console.log('Tauri detection via __TAURI__ fallback:', isTauriApp);
		}
		
		if (!isTauriApp) {
			// Try alternative detection method
			try {
				// Try to import and use Tauri API to confirm we're in Tauri
				const { invoke } = await import('@tauri-apps/api/core');
				await invoke('check_server_status');
				isTauriApp = true;
				console.log('Detected Tauri via API test');
			} catch (e) {
				console.log('Tauri API test failed, running in web mode:', e);
				isTauriApp = false;
			}
		}
		
		if (!isTauriApp) {
			// In web mode, assume server is available (for development)
			isServerReady = true;
			notifications.info('Welcome to Karta! (Web Mode)', 2000);
		} else {
			console.log('Running in Tauri mode - showing vault selection');
		}
		// In Tauri mode, ServerSetupModal will handle server startup
	});

	function handleServerReady() {
		isServerReady = true;
		notifications.info('Welcome to Karta!', 2000);
	}
</script>

{#if isTauriApp}
	<ServerSetupModal onServerReady={handleServerReady} />
{/if}

{#if isServerReady}
	<div class="w-full min-h-screen bg-gray-400 flex flex-col relative">
		<AppMenu />
		<Toolbar />
		<Viewport />
		<ContextPathDisplay />
		<!--<KartaDebugOverlay /> -->
		<PropertiesPanel />
		<Notification />
		<ColorPickerPopup />
		<div id="portal-container"></div>
	</div>
{/if}
