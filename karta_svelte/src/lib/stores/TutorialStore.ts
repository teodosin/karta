import { writable } from 'svelte/store';

export const isTutorialOpen = writable(false);

export function openTutorial() {
	isTutorialOpen.set(true);
}

export function closeTutorial() {
	isTutorialOpen.set(false);
}
