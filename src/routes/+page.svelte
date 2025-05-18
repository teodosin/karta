<script lang="ts">
	import Viewport from '$lib/components/Viewport.svelte';
	import Toolbar from '$lib/components/Toolbar.svelte';
	import ContextPathDisplay from '$lib/components/ContextPathDisplay.svelte';
	import KartaDebugOverlay from '$lib/components/KartaDebugOverlay.svelte';
	import PropertiesPanel from '$lib/components/PropertiesPanel.svelte';
	import AppMenu from '$lib/components/AppMenu.svelte';
</script>

<div class="w-full min-h-screen bg-gray-100 flex flex-col relative">
	<AppMenu />
	<Toolbar />
	<Viewport />
	<ContextPathDisplay />
	<!--<KartaDebugOverlay /> -->
	<PropertiesPanel />
</div>
