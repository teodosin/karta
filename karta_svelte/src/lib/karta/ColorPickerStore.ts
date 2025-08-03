import { writable, get } from 'svelte/store';
import { settings, updateSettings } from '$lib/karta/SettingsStore';
import type { ColorTheme } from '$lib/types/types';

type ColorPickerState = {
	isOpen: boolean;
	initialColor: string;
	currentColor: string;
	position: { x: number; y: number };
	onUpdate: (color: string) => void;
	onClose?: (finalColor: string) => void;
};

const initialState: ColorPickerState = {
	isOpen: false,
	initialColor: '#ffffff',
	currentColor: '#ffffff',
	position: { x: 0, y: 0 },
	onUpdate: () => {},
	onClose: undefined
};

const { subscribe, update } = writable<ColorPickerState>(initialState);

function open(initialColor: string, event: MouseEvent, onUpdate: (color: string) => void, onClose?: (finalColor: string) => void) {
	const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
	
	// Calculate position with viewport bounds
	const viewportWidth = window.innerWidth;
	const viewportHeight = window.innerHeight;
	const pickerWidth = 280; // Approximate width of color picker
	const pickerHeight = 350; // Approximate height of color picker
	
	let x = rect.left;
	let y = rect.bottom + 5;
	
	// Keep within right boundary
	if (x + pickerWidth > viewportWidth) {
		x = viewportWidth - pickerWidth - 10;
	}
	
	// Keep within left boundary
	if (x < 10) {
		x = 10;
	}
	
	// Check if picker would go below viewport
	if (y + pickerHeight > viewportHeight) {
		// Position above the button instead
		y = rect.top - pickerHeight - 5;
		
		// If still outside viewport, position at top
		if (y < 10) {
			y = 10;
		}
	}
	
	update((state) => ({
		...state,
		isOpen: true,
		initialColor,
		currentColor: initialColor,
		position: { x, y },
		onUpdate,
		onClose
	}));
}

let currentState: ColorPickerState = initialState;
// Track state for internal use
const storeInstance = { subscribe, update };
storeInstance.subscribe(state => {
	currentState = state;
});

function updateColor(color: string) {
	update((state) => ({
		...state,
		currentColor: color
	}));
	
	// Call the immediate update callback for visual feedback
	if (currentState.onUpdate) {
		currentState.onUpdate(color);
	}
}

function close() {
	if (currentState.onClose) {
		currentState.onClose(currentState.currentColor);
	}
	update((state) => ({ ...state, isOpen: false, onClose: undefined }));
}

export const colorPickerStore = {
	subscribe,
	open,
	close,
	updateColor
};