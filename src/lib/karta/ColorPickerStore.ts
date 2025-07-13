import { writable, get } from 'svelte/store';
import { settings, updateSettings } from '$lib/karta/SettingsStore';
import type { ColorTheme } from '$lib/types/types';

type ColorPickerState = {
	isOpen: boolean;
	initialColor: string;
	position: { x: number; y: number };
	onUpdate: (color: string) => void;
};

const initialState: ColorPickerState = {
	isOpen: false,
	initialColor: '#ffffff',
	position: { x: 0, y: 0 },
	onUpdate: () => {}
};

const { subscribe, update } = writable<ColorPickerState>(initialState);

function open(initialColor: string, event: MouseEvent, onUpdate: (color: string) => void) {
	const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
	update((state) => ({
		...state,
		isOpen: true,
		initialColor,
		position: { x: rect.left, y: rect.bottom + 5 },
		onUpdate
	}));
}

function close() {
	update((state) => ({ ...state, isOpen: false }));
}

export const colorPickerStore = {
	subscribe,
	open,
	close
};