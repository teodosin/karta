import { writable } from 'svelte/store';
import { v4 as uuidv4 } from 'uuid';

export type NotificationType = 'success' | 'error' | 'info';

export interface Notification {
	id: string;
	type: NotificationType;
	message: string;
	duration: number;
}

const { subscribe, update } = writable<Notification[]>([]);

function show(message: string, type: NotificationType = 'info', duration: number = 3000) {
	const id = uuidv4();
	const notification: Notification = {
		id,
		type,
		message,
		duration
	};

	update((notifications) => [...notifications, notification]);

	setTimeout(() => {
		remove(id);
	}, duration);
}

function remove(id: string) {
	update((notifications) => notifications.filter((n) => n.id !== id));
}

export const notifications = {
	subscribe,
	show,
	remove,
	success: (message: string, duration?: number) => show(message, 'success', duration),
	error: (message: string, duration?: number) => show(message, 'error', duration),
	info: (message: string, duration?: number) => show(message, 'info', duration)
};