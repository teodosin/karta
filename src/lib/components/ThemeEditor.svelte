<script lang="ts">
	import { settings, updateSettings } from '$lib/karta/SettingsStore';
	import { colorPickerStore } from '$lib/karta/ColorPickerStore';
	import type { ColorTheme } from '$lib/types/types';

	const themeColors: { key: keyof ColorTheme; label: string }[] = [
		{ key: 'viewport-bg', label: 'Viewport Background' },
		{ key: 'panel-bg', label: 'Panel Background' },
		{ key: 'focal-hl', label: 'Focal Highlight' },
		{ key: 'panel-hl', label: 'Panel Highlight' },
		{ key: 'text-color', label: 'Text Color' }
	];

	function handleOpenPicker(key: keyof ColorTheme, e: MouseEvent) {
		const initialColor = $settings.colorTheme[key];
		const onUpdate = (newColor: string) => {
			const newTheme = { ...$settings.colorTheme, [key]: newColor };
			updateSettings({ colorTheme: newTheme });
		};
		colorPickerStore.open(initialColor, e, onUpdate);
	}

	function resetToDefaults() {
		if (confirm('Are you sure you want to reset the theme to defaults?')) {
			updateSettings({ colorTheme: settings.defaultSettings.colorTheme });
		}
	}
</script>

<div class="p-4 space-y-4">
	<h3 class="text-lg font-semibold">Theme Colors</h3>
	<div class="space-y-2">
		{#each themeColors as color}
			<div class="flex items-center justify-between">
				<label class="text-sm">{color.label}</label>
				<button
					class="w-8 h-5 rounded border border-gray-400"
					style="background-color: {$settings.colorTheme[color.key]}"
					on:click={(e) => handleOpenPicker(color.key, e)}
				></button>
			</div>
		{/each}
	</div>
	<div class="pt-2">
		<button
			class="w-full px-4 py-1 text-sm font-medium text-white bg-gray-800 rounded-md hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500"
			on:click={resetToDefaults}
		>
			Reset to Defaults
		</button>
	</div>
</div>
