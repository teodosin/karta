import { writable } from 'svelte/store';

// Master switch for all debugging. Can be controlled from anywhere in the app.
export const isDebugMode = writable(false);