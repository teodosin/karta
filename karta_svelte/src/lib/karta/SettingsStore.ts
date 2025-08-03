import { writable } from 'svelte/store';
import type { KartaSettings, ColorTheme } from '$lib/types/types';
import { ServerAdapter } from '$lib/util/ServerAdapter';

const adapter = new ServerAdapter();

// Default settings
const defaultSettings: KartaSettings = {
	version: 0.1,
	savelastViewedContextPath: true,
	lastViewedContextPath: null,
	colorTheme: {
		'viewport-bg': '#000000',
		'panel-bg': '#431d1f',
		'focal-hl': '#f4902dff',
		'panel-hl': '#741f2fff',
		'text-color': '#f0f0f0',
		'contrast-color': '#60a5fa',
		'connection-color': '#505050'
	},
	edgeFilters: {
		containsEdges: 'always',
		normalEdges: 'always'
	}
};

// Create the writable store, initialized with defaults
const { subscribe, set, update } = writable<KartaSettings>(defaultSettings);

// Function to load settings from the server
async function loadSettings() {
	try {
		const serverSettings = await adapter.getSettings();
		if (serverSettings) {
			// Deep merge loaded settings with defaults to ensure all keys exist
			const mergedSettings = {
				...defaultSettings,
				...serverSettings,
				colorTheme: { ...defaultSettings.colorTheme, ...serverSettings.colorTheme },
				edgeFilters: { ...defaultSettings.edgeFilters, ...serverSettings.edgeFilters }
			};
			set(mergedSettings);
		} else {
			// No settings found on server, save the defaults
			await adapter.saveSettings(defaultSettings);
			set(defaultSettings);
		}
	} catch (error) {
		console.error('Error loading Karta settings from server:', error);
		// Fallback to default settings if loading fails
		set(defaultSettings);
	}
}

// Function to update settings and save to the server
export async function updateSettings(newSettings: Partial<KartaSettings>) {
	let updatedSettings: KartaSettings | undefined;
	update((currentSettings) => {
		updatedSettings = { ...currentSettings, ...newSettings };
		return updatedSettings;
	});

	if (updatedSettings) {
		try {
			await adapter.saveSettings(updatedSettings);
		} catch (error) {
			console.error('Error saving Karta settings to server:', error);
		}
	}
}

// Export the store interface
export const settings = {
	subscribe,
	loadSettings, // Expose load function to be called on app init
	updateSettings, // Expose update function
	defaultSettings // Expose default settings for reset functionality
};