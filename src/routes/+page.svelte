<script lang="ts">
	import Viewport from '$lib/components/Viewport.svelte';
	import Toolbar from '$lib/components/Toolbar.svelte';
	import ContextPathDisplay from '$lib/components/ContextPathDisplay.svelte'; // Import the component
	import KartaDebugOverlay from '$lib/components/KartaDebugOverlay.svelte'; // Import the debug overlay
	import PropertiesPanel from '$lib/components/PropertiesPanel.svelte'; // Import the properties panel
</script>

<div class="w-full min-h-screen bg-gray-100 flex flex-col relative"> <!-- Added relative positioning -->
	<Toolbar />
	<Viewport />
	<ContextPathDisplay />
	<!--<KartaDebugOverlay /> -->
	<PropertiesPanel />
</div>
