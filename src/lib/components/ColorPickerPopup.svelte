<script lang="ts">
	import { colorPickerStore } from '$lib/karta/ColorPickerStore';
	import ColorPicker from 'svelte-awesome-color-picker';
	import { settings } from '$lib/karta/SettingsStore';
	import { colord } from 'colord';

	function handleColorInput(data: { rgb: { r: number; g: number; b: number; a: number } | null }) {
		if (data.rgb && $colorPickerStore.onUpdate) {
			const newColor = `rgba(${data.rgb.r}, ${data.rgb.g}, ${data.rgb.b}, ${data.rgb.a})`;
			$colorPickerStore.onUpdate(newColor);
		}
	}
</script>

{#if $colorPickerStore.isOpen}
	<div
		class="fixed inset-0 z-[999]"
		on:click={() => colorPickerStore.close()}
		on:contextmenu|preventDefault
	></div>
	<div
		class="fixed z-[1000] bg-transparent shadow-lg p-2"
		style="left: {$colorPickerStore.position.x}px; top: {$colorPickerStore.position.y}px;"
	>
		<ColorPicker
			rgb={colord($colorPickerStore.initialColor).toRgb()}
			onInput={handleColorInput}
			isDialog={false}
		/>
	</div>
{/if}