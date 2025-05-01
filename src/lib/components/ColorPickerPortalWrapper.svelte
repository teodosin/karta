<script lang="ts">
	import { portal } from "svelte-portal";

	let { wrapper = $bindable(), isOpen, isDialog, children } = $props();
</script>

<div
	use:portal={"#portal"}
	bind:this={wrapper}
	class="wrapper"
	class:is-open={isOpen}
	role={isDialog ? 'dialog' : undefined}
	aria-label="color picker"
>
	{@render children()}
</div>

<style>
	div {
		/* Base styles copied from example */
		padding: 8px;
		background-color: var(--cp-bg-color, white);
		margin: 0 10px 10px; /* Restore margin */
		border: 1px solid var(--cp-border-color, black);
		border-radius: 12px;
		display: none;
		width: max-content;
		/* Ensure it appears above other elements */
		/* position: absolute; /* Let the color picker library handle positioning */ 
		z-index: 50; /* Match z-index of portal target or higher */
	}
	.is-open {
		display: inline-block;
	}
	/* Removed [role='dialog'] specific positioning as portal handles placement */
</style>