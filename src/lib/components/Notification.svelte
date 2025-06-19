<script lang="ts">
	import { notifications, type Notification } from '$lib/karta/NotificationStore';
	import { fly, slide } from 'svelte/transition';

	export let fadeOutDuration: number = 300;
	export let maxVisibleNotifications: number = 5;

	let currentNotifications: Notification[] = [];

	notifications.subscribe((value) => {
		currentNotifications = value.slice(-maxVisibleNotifications);
	});
</script>

<div
	class="fixed bottom-4 right-4 z-50 flex flex-col items-end space-y-2 pointer-events-none"
	aria-live="assertive"
>
	{#each currentNotifications as notification (notification.id)}
		<div
			in:fly|local={{ y: 20, duration: 300 }}
			out:slide|local={{ duration: fadeOutDuration }}
			class="w-auto max-w-sm rounded-md p-3 shadow-lg text-sm font-medium pointer-events-auto {notification.type ===
			'success'
				? 'bg-green-500 text-white'
				: notification.type === 'error'
				? 'bg-red-500 text-white'
				: 'bg-gray-800 text-white'}"
		>
			{notification.message}
		</div>
	{/each}
</div>