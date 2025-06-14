import { writable } from 'svelte/store';
import type { KartaSettings } from '$lib/types/types';

const SETTINGS_STORAGE_KEY = 'kartaSettings';

// Default settings
const defaultSettings: KartaSettings = {
	version: 1,
	saveLastViewedContext: true,
	vaultPath: null
};

// Create the writable store, initialized with defaults
const { subscribe, set, update } = writable<KartaSettings>(defaultSettings);

// Function to load settings from localStorage and merge with defaults
function loadSettings() {
	try {
		const storedSettingsJson = localStorage.getItem(SETTINGS_STORAGE_KEY);
		if (storedSettingsJson) {
			const storedSettings = JSON.parse(storedSettingsJson) as Partial<KartaSettings>;
			// Merge loaded settings with defaults to ensure all keys exist
			// and new defaults are applied if the stored version is older
			const mergedSettings = { ...defaultSettings, ...storedSettings };
			set(mergedSettings); // Update the store with merged settings
		} else {
			// No settings found, save the defaults to localStorage
			localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(defaultSettings));
			set(defaultSettings); // Ensure store has defaults
		}
	} catch (error) {
		console.error('Error loading Karta settings from localStorage:', error);
		// Fallback to default settings if loading/parsing fails
		set(defaultSettings);
		// Optionally clear corrupted storage
		// localStorage.removeItem(SETTINGS_STORAGE_KEY);
	}
}

// Function to update settings and save to localStorage
function updateSettings(newSettings: Partial<KartaSettings>) {
	update((currentSettings) => {
		const updatedSettings = { ...currentSettings, ...newSettings };
		try {
			localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(updatedSettings));
		} catch (error) {
			console.error('Error saving Karta settings to localStorage:', error);
			// Decide if we should revert the store update or just log the error
		}
		return updatedSettings;
	});
}

// Export the store interface
export const settings = {
	subscribe,
	loadSettings, // Expose load function to be called on app init
	updateSettings // Expose update function
	// Note: Direct 'set' is not exposed to enforce saving via updateSettings
};

// Initial load when the module is first imported (usually on app startup)
// loadSettings(); // Moved call to initialization logic