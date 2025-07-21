<script lang="ts">
	import { colorPickerStore } from '$lib/karta/ColorPickerStore';
	import ColorPicker from 'svelte-awesome-color-picker';
	import { colord } from 'colord';
	import { onMount } from 'svelte';

	let popupElement: HTMLDivElement;
	let popupWidth = 0;
	let popupHeight = 0;

	function handleColorInput(data: { rgb: { r: number; g: number; b: number; a: number } | null }) {
		if (data.rgb) {
			const newColor = `rgba(${data.rgb.r}, ${data.rgb.g}, ${data.rgb.b}, ${data.rgb.a})`;
			colorPickerStore.updateColor(newColor);
		}
	}

	$: if (popupElement) {
		popupWidth = popupElement.offsetWidth;
		popupHeight = popupElement.offsetHeight;
	}

	$: adjustedPosition = (() => {
		if (!$colorPickerStore.isOpen) return { x: 0, y: 0 };

		let x = $colorPickerStore.position.x;
		let y = $colorPickerStore.position.y;

		if (typeof window !== 'undefined') {
			if (x + popupWidth > window.innerWidth) {
				x = window.innerWidth - popupWidth - 10;
			}
			if (y + popupHeight > window.innerHeight) {
				y = window.innerHeight - popupHeight - 10;
			}
		}

		return { x, y };
	})();
</script>

{#if $colorPickerStore.isOpen}
	<div
		class="fixed inset-0 z-[999]"
		on:click={() => colorPickerStore.close()}
		on:contextmenu|preventDefault
	></div>
	<div
		bind:this={popupElement}
		class="fixed z-[1000] p-2"
		style="left: {adjustedPosition.x}px; top: {adjustedPosition.y}px;"
	>
		<ColorPicker
			rgb={colord($colorPickerStore.currentColor).toRgb()}
			onInput={handleColorInput}
			isDialog={false}
		/>
	</div>
{/if}