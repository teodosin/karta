<script lang="ts">
	import Viewport from '$lib/components/Viewport.svelte';
	import Toolbar from '$lib/components/Toolbar.svelte';
	import ContextPathDisplay from '$lib/components/ContextPathDisplay.svelte'; // Import the component
</script>

<div class="w-full min-h-screen bg-gray-100 flex flex-col relative"> <!-- Added relative positioning -->
	<Toolbar />
	<Viewport />
	<ContextPathDisplay /> <!-- Add the component -->
</div>
