<script lang="ts">
	import { onMount } from 'svelte';
	import Viewport from '$lib/components/Viewport.svelte';
	import Toolbar from '$lib/components/Toolbar.svelte';
	import ContextPathDisplay from '$lib/components/ContextPathDisplay.svelte';
	import KartaDebugOverlay from '$lib/components/KartaDebugOverlay.svelte';
	import PropertiesPanel from '$lib/components/PropertiesPanel.svelte';
	import AppMenu from '$lib/components/AppMenu.svelte';
	import Notification from '$lib/components/Notification.svelte';
	import { notifications } from '$lib/karta/NotificationStore';
	import ColorPickerPopup from '$lib/components/ColorPickerPopup.svelte';

	onMount(() => {
		// Show a test notification when the page is ready
		notifications.info('Welcome to Karta!', 2000);
	});
</script>

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
