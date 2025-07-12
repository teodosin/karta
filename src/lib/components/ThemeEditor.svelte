<script lang="ts">
	import { settings, updateSettings } from '$lib/karta/SettingsStore';
	import ColorPicker from 'svelte-awesome-color-picker';
	import ColorPickerPortalWrapper from './ColorPickerPortalWrapper.svelte';
	import { colord, type Colord } from 'colord';

	function handleThemeColorChange(key: 'viewport-bg' | 'panel-bg' | 'focal-hl' | 'panel-hl', data: { rgb: { r: number; g: number; b: number; a: number; } | null }) {
		if (data.rgb) {
			const newColor = `rgba(${data.rgb.r}, ${data.rgb.g}, ${data.rgb.b}, ${data.rgb.a})`;
			updateSettings({
				colorTheme: {
					...$settings.colorTheme,
					[key]: newColor
				}
			});
		}
	}
</script>

<div class="p-4 space-y-4">
	<h3 class="text-lg font-semibold">Theme Colors</h3>
	<div class="space-y-2">
		<!-- Viewport Background -->
		<div class="flex items-center justify-between">
			<label for="viewport-bg-color" class="text-sm">Viewport Background</label>
			<ColorPicker
				rgb={colord($settings.colorTheme['viewport-bg']).toRgb()}
				onInput={(e) => handleThemeColorChange('viewport-bg', e)}
				position="responsive"
				components={{ wrapper: ColorPickerPortalWrapper }}
				disableCloseClickOutside={true}
			/>
		</div>

		<!-- Panel Background -->
		<div class="flex items-center justify-between">
			<label for="panel-bg-color" class="text-sm">Panel Background</label>
			<ColorPicker
				rgb={colord($settings.colorTheme['panel-bg']).toRgb()}
				onInput={(e) => handleThemeColorChange('panel-bg', e)}
				position="responsive"
				components={{ wrapper: ColorPickerPortalWrapper }}
				disableCloseClickOutside={true}
			/>
		</div>

		<!-- Focal Highlight -->
		<div class="flex items-center justify-between">
			<label for="focal-hl-color" class="text-sm">Focal Highlight</label>
			<ColorPicker
				rgb={colord($settings.colorTheme['focal-hl']).toRgb()}
				onInput={(e) => handleThemeColorChange('focal-hl', e)}
				position="responsive"
				components={{ wrapper: ColorPickerPortalWrapper }}
				disableCloseClickOutside={true}
			/>
		</div>

		<!-- Panel Highlight -->
		<div class="flex items-center justify-between">
			<label for="panel-hl-color" class="text-sm">Panel Highlight</label>
			<ColorPicker
				rgb={colord($settings.colorTheme['panel-hl']).toRgb()}
				onInput={(e) => handleThemeColorChange('panel-hl', e)}
				position="responsive"
				components={{ wrapper: ColorPickerPortalWrapper }}
				disableCloseClickOutside={true}
			/>
		</div>
	</div>
</div>