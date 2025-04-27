<script lang="ts">
	import { isConfirmationDialogOpen, confirmationDialogMessage, confirmationDialogAction } from '$lib/karta/UIStateStore';

	// Action to close the dialog
	function closeDialog() {
		isConfirmationDialogOpen.set(false);
		confirmationDialogMessage.set('');
		confirmationDialogAction.set(null); // Clear the action
	}

	// Action to confirm and execute the stored action
	function confirmAction() {
		const action = $confirmationDialogAction;
		if (action) {
			action(); // Execute the stored action
		}
		closeDialog(); // Close the dialog after action
	}

	// Handle Escape key to close dialog
	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			closeDialog();
		}
	}
</script>

{#if $isConfirmationDialogOpen}
<script>console.log('ConfirmationDialog is open');</script>
<div
	class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-[100]"
	role="dialog"
	aria-modal="true"
	aria-labelledby="confirmation-dialog-title"
	on:keydown={handleKeyDown}
	tabindex="-1"
>
	<div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl p-6 max-w-sm w-full">
		<h3 id="confirmation-dialog-title" class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
			Confirm Action
		</h3>
		<div class="text-sm text-gray-700 dark:text-gray-300 mb-6">
			{$confirmationDialogMessage}
		</div>
		<div class="flex justify-end space-x-4">
			<button
				type="button"
				class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-200 bg-gray-200 dark:bg-gray-700 rounded-md hover:bg-gray-300 dark:hover:bg-gray-600 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
				on:click={closeDialog}
			>
				Cancel
			</button>
			<button
				type="button"
				class="px-4 py-2 text-sm font-medium text-white bg-red-600 rounded-md hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500"
				on:click={confirmAction}
			>
				Confirm
			</button>
		</div>
	</div>
</div>
{/if}

<style>
	/* Add any specific styles if needed, Tailwind covers most */
</style>