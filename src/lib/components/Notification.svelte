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
	class="fixed bottom-4 right-4 z-50 flex flex-col items-end pointer-events-none"
	aria-live="assertive"
>
	{#each currentNotifications as notification (notification.id)}
		<div
			in:fly|local={{ y: 20, duration: 300 }}
			out:slide|local={{ duration: fadeOutDuration }}
			class="w-auto max-w-sm rounded-md p-2 text-xs font-medium pointer-events-auto bg-opacity-80 {notification.type ===
			'success'
				? 'text-green-300'
				: notification.type === 'error'
				? 'text-red-300'
				: 'text-gray-100'}"
			style="background-color: var(--color-viewport-bg, #2b2b36); opacity: 0.7;"
		>
			{notification.message}
		</div>
	{/each}
</div>