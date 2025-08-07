<script lang="ts">
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import { serverManager } from '$lib/tauri/server';
	import VaultSelectionModal from './VaultSelectionModal.svelte';
	import { fade } from 'svelte/transition';
	import { Server, CheckCircle, AlertTriangle } from 'lucide-svelte';
	import { initializeVault } from '$lib/karta/VaultStore';
	import { initializeStores } from '$lib/karta/ContextStore';

	export let onServerReady: () => void = () => {};

	let currentStep: 'checking' | 'vault-selection' | 'starting-server' | 'ready' | 'error' = 'checking';
	let errorMessage = '';
	let isVaultSelectionOpen = false;
	let selectedVaultPath = '';

	onMount(async () => {
		if (browser) {
			await checkServerStatus();
		}
	});

	async function checkServerStatus() {
		try {
			currentStep = 'checking';
			const isRunning = await serverManager.checkServerStatus();
			
			if (isRunning) {
				currentStep = 'ready';
				onServerReady();
			} else {
				currentStep = 'vault-selection';
				isVaultSelectionOpen = true;
			}
		} catch (error) {
			currentStep = 'error';
			errorMessage = error instanceof Error ? error.message : 'Failed to check server status';
		}
	}

	async function handleVaultSelected(vaultPath: string) {
		selectedVaultPath = vaultPath;
		isVaultSelectionOpen = false;
		await startServer(vaultPath);
	}

	async function startServer(vaultPath: string) {
		try {
			currentStep = 'starting-server';
			await serverManager.startServer(vaultPath);
			
			// Initialize stores after server is ready
			await initializeVault();
			await initializeStores();
			
			currentStep = 'ready';
			onServerReady();
		} catch (error) {
			currentStep = 'error';
			errorMessage = error instanceof Error ? error.message : 'Failed to start server';
		}
	}

	function handleVaultSelectionClose() {
		isVaultSelectionOpen = false;
		// If user closes without selecting, go back to vault selection
		if (currentStep === 'vault-selection') {
			currentStep = 'vault-selection';
			isVaultSelectionOpen = true;
		}
	}

	function retry() {
		errorMessage = '';
		checkServerStatus();
	}

	function selectDifferentVault() {
		currentStep = 'vault-selection';
		isVaultSelectionOpen = true;
	}
</script>

<!-- Server setup overlay -->
{#if currentStep !== 'ready' && currentStep !== 'vault-selection'}
	<div class="fixed inset-0 z-50 flex items-center justify-center" style="background-color: var(--color-viewport-bg);">
		<div 
			class="bg-panel-bg border border-gray-700 rounded-lg shadow-2xl p-6 w-[400px] max-w-[90vw]"
			transition:fade={{ duration: 300 }}
		>
			<div class="flex items-center gap-3 mb-4">
				<Server size={24} style="color: var(--color-text-color);" />
				<h1 class="text-xl font-semibold" style="color: var(--color-text-color);">
					Karta Server
				</h1>
			</div>

			{#if currentStep === 'checking'}
				<div class="text-center py-6">
					<div class="animate-spin text-2xl mb-3">⟳</div>
					<p class="text-sm" style="color: var(--color-text-color);">
						Checking server status...
					</p>
				</div>

			{:else if currentStep === 'starting-server'}
				<div class="text-center py-6">
					<div class="animate-spin text-2xl mb-3">⟳</div>
					<p class="text-sm mb-2" style="color: var(--color-text-color);">
						Starting server...
					</p>
					<p class="text-xs opacity-70" style="color: var(--color-text-color);">
						Vault: {selectedVaultPath}
					</p>
				</div>

			{:else if currentStep === 'error'}
				<div class="text-center py-6">
					<AlertTriangle size={32} class="mx-auto mb-3 text-red-500" />
					<p class="text-sm font-medium mb-2 text-red-400">
						Server Error
					</p>
					<p class="text-xs mb-4 opacity-70" style="color: var(--color-text-color);">
						{errorMessage}
					</p>
					<div class="flex gap-2 justify-center">
						<button
							on:click={retry}
							class="px-3 py-1.5 text-xs bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
						>
							Retry
						</button>
						{#if selectedVaultPath}
							<button
								on:click={selectDifferentVault}
								class="px-3 py-1.5 text-xs border border-gray-600 rounded hover:border-gray-500 transition-colors"
								style="color: var(--color-text-color);"
							>
								Different Vault
							</button>
						{/if}
					</div>
				</div>

			{/if}
		</div>
	</div>
{/if}

<!-- Vault selection modal -->
<VaultSelectionModal
	isOpen={isVaultSelectionOpen}
	onVaultSelected={handleVaultSelected}
	onClose={handleVaultSelectionClose}
/>

<style>
	.bg-panel-bg {
		background-color: var(--color-panel-bg);
	}
</style>
