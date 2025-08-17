<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { browser } from '$app/environment';
	import { fade, scale } from 'svelte/transition';
	import { HardDrive, Plus, AlertTriangle, CheckCircle, Folder } from 'lucide-svelte';
	import { serverManager, type VaultInfo } from '$lib/tauri/server';
	import { invoke } from '@tauri-apps/api/core';
	// OS detection for conditional macOS behavior in Tauri without importing @tauri-apps/api/os
	let isTauriMac = false;

	onMount(async () => {
		if (browser && '__TAURI__' in window) {
			// Use UA/platform to detect macOS to avoid bundling @tauri-apps/api/os
			const ua = navigator.userAgent || '';
			const plat = (navigator as any).platform || '';
			isTauriMac = /Macintosh|Mac OS X/i.test(ua) || /Mac/i.test(plat);
		}
	});

	export let isOpen = false;
	export let onVaultSelected: (vaultPath: string) => void = () => {};
	export let onClose: () => void = () => {};

	let vaults: VaultInfo[] = [];
	let selectedIndex = -1;
	let isLoading = false;
	let error = '';
	let modalElement: HTMLDivElement;

	// Load available vaults when modal opens
	async function loadVaults() {
		try {
			isLoading = true;
			error = '';
			vaults = await serverManager.getAvailableVaults();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load vaults';
			vaults = [];
		} finally {
			isLoading = false;
		}
	}

	// Handle vault selection
	async function selectVault(vaultPath: string) {
		onVaultSelected(vaultPath);
		closeModal();
	}

	// macOS sandbox: silent bookmark-based access if possible; otherwise one-time capture
	async function selectVaultWithAccess(vaultPath: string) {
		if (isTauriMac) {
			try {
				// First, try existing bookmark to enable access silently
				const hasAccess = await invoke<boolean>('ensure_vault_access', { path: vaultPath });
				if (hasAccess) {
					await selectVault(vaultPath);
					return;
				}
				// No bookmark yet: capture one using the directory picker, then save bookmark data
				const { open } = await import('@tauri-apps/plugin-dialog');
				const selectedPath = await open({ directory: true, multiple: false, defaultPath: vaultPath });
				if (!selectedPath) return; // canceled
				if (selectedPath !== vaultPath) {
					await serverManager.addVaultToConfig(selectedPath);
					vaultPath = selectedPath as string;
				}
				await invoke('save_vault_bookmark_from_path', { path: vaultPath });
				const granted = await invoke<boolean>('ensure_vault_access', { path: vaultPath });
				if (granted) {
					await selectVault(vaultPath);
				}
			} catch (err) {
				error = err instanceof Error ? err.message : 'Failed to authorize directory access';
			}
			return;
		}
		await selectVault(vaultPath);
	}

	// Handle directory selection
	async function selectNewDirectory() {
		try {
			// Use the Tauri dialog plugin to open a directory picker
			const { open } = await import('@tauri-apps/plugin-dialog');
			const selectedPath = await open({
				directory: true,
				multiple: false,
			});
			
			if (selectedPath) {
				// Add to config and then select it
				await serverManager.addVaultToConfig(selectedPath as string);
				await selectVault(selectedPath as string);
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to select directory';
		}
	}

	// Close modal
	function closeModal() {
		selectedIndex = -1;
		error = '';
		onClose();
	}

	// Keyboard navigation
	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			closeModal();
		} else if (event.key === 'ArrowDown') {
			event.preventDefault();
			selectedIndex = Math.min(selectedIndex + 1, vaults.length); // +1 for "Browse..." option
		} else if (event.key === 'ArrowUp') {
			event.preventDefault();
			selectedIndex = Math.max(selectedIndex - 1, 0);
		} else if (event.key === 'Enter') {
			event.preventDefault();
			if (selectedIndex === vaults.length) {
				// "Browse..." is selected
				selectNewDirectory();
			} else if (selectedIndex >= 0 && selectedIndex < vaults.length) {
				const vault = vaults[selectedIndex];
				if (vault.exists && vault.has_karta_dir) {
					selectVaultWithAccess(vault.path);
				}
			}
		}
	}

	// Click outside handler
	function handleClickOutside(event: MouseEvent) {
		if (isOpen && modalElement && !modalElement.contains(event.target as Node)) {
			// Don't close here since backdrop handles it
		}
	}

	onMount(() => {
		if (browser) {
			window.addEventListener('pointerdown', handleClickOutside, true);
		}
	});

	// Load vaults when modal opens
	$: if (isOpen) {
		selectedIndex = -1;
		loadVaults();
	}

	onDestroy(() => {
		if (browser) {
			window.removeEventListener('pointerdown', handleClickOutside, true);
		}
	});
</script>

{#if isOpen}
	<!-- Transparent backdrop for click-to-close -->
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div 
		class="fixed inset-0 z-40"
		transition:fade={{ duration: 200 }}
		on:click={closeModal}
	></div>

	<!-- Vault selection modal -->
	<div
		bind:this={modalElement}
		class="fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 z-50 
		       border border-gray-700 rounded-lg shadow-2xl w-[600px] max-w-[90vw] h-[500px] flex flex-col"
		style="background-color: color-mix(in srgb, var(--color-panel-bg) 80%, transparent);"
		role="dialog"
		aria-modal="true"
		aria-labelledby="vault-selection-label"
		transition:scale={{ duration: 200, start: 0.95 }}
		on:keydown={handleKeyDown}
		tabindex="-1"
	>
		<!-- Modal header -->
		<div class="p-4 border-b border-gray-700">
			<div class="flex items-center gap-2">
				<HardDrive size={20} style="color: var(--color-text-color);" />
				<h2 id="vault-selection-label" class="text-lg font-medium" style="color: var(--color-text-color);">
					Select Vault
				</h2>
			</div>
			<p class="text-sm mt-2 opacity-80" style="color: var(--color-text-color);">
				Choose an existing vault or create a new one. Karta will only have access to files inside of a vault. This keeps your workspace isolated and secure.
			</p>
		</div>

		<!-- Content area -->
		<div class="flex-1 overflow-hidden p-4">
			{#if isLoading}
				<div class="flex items-center justify-center h-full">
					<div class="text-gray-400">
						<div class="animate-spin text-lg mb-2">⟳</div>
						<div class="text-sm font-medium">Loading vaults...</div>
					</div>
				</div>
			{:else if error}
				<div class="flex items-center justify-center h-full">
					<div class="text-red-400 text-center">
						<AlertTriangle size={32} class="mx-auto mb-2" />
						<div class="text-sm font-medium mb-1">Error</div>
						<div class="text-xs">{error}</div>
					</div>
				</div>
			{:else}
				<div class="h-full overflow-y-auto space-y-2">
					<!-- Existing vaults -->
					{#each vaults as vault, index (vault.path)}
						<!-- svelte-ignore a11y_click_events_have_key_events -->
						<div
							class="p-3 rounded-lg border cursor-pointer transition-colors
							       {selectedIndex === index
								? 'border-blue-500 hover-bg-panel-hl'
								: vault.exists && vault.has_karta_dir
									? 'border-gray-600 hover:border-gray-500 hover:opacity-80'
									: 'border-red-600 opacity-50 cursor-not-allowed'}"
							style="background-color: color-mix(in srgb, var(--color-panel-bg) 40%, transparent);"
							on:click={() => vault.exists && vault.has_karta_dir && selectVaultWithAccess(vault.path)}
							on:mouseenter={() => vault.exists && vault.has_karta_dir && (selectedIndex = index)}
							role="button"
							tabindex="-1"
						>
							<div class="flex items-center gap-3">
								<!-- Status icon -->
								<div class="flex-shrink-0">
									{#if vault.exists && vault.has_karta_dir}
										<CheckCircle size={20} class="text-green-500" />
									{:else if vault.exists && !vault.has_karta_dir}
										<AlertTriangle size={20} class="text-yellow-500" />
									{:else}
										<AlertTriangle size={20} class="text-red-500" />
									{/if}
								</div>

								<!-- Vault info -->
								<div class="flex-1 min-w-0">
									<div class="font-mono text-sm truncate" style="color: var(--color-text-color);">
										{vault.path}
									</div>
									<div class="text-xs mt-1" style="color: var(--color-text-color); opacity: 0.7;">
										{#if vault.exists && vault.has_karta_dir}
											<span class="text-green-400">Ready</span>
										{:else if vault.exists && !vault.has_karta_dir}
											<span class="text-yellow-400">Missing .karta directory</span>
										{:else}
											<span class="text-red-400">Directory does not exist</span>
										{/if}
									</div>
								</div>
							</div>
						</div>
					{/each}

					<!-- Browse for new directory option -->
					<!-- svelte-ignore a11y_click_events_have_key_events -->
					<div
						class="p-3 rounded-lg border border-dashed cursor-pointer transition-colors
						       {selectedIndex === vaults.length
							? 'border-blue-500 hover-bg-panel-hl'
							: 'border-gray-600 hover:border-gray-500 hover:opacity-80'}"
						style="background-color: color-mix(in srgb, var(--color-panel-bg) 20%, transparent);"
						on:click={selectNewDirectory}
						on:mouseenter={() => (selectedIndex = vaults.length)}
						role="button"
						tabindex="-1"
					>
						<div class="flex items-center gap-3">
							<div class="flex-shrink-0">
								<Plus size={20} style="color: var(--color-contrast-color);" />
							</div>
							<div class="flex-1">
								<div class="font-medium text-sm" style="color: var(--color-contrast-color);">
									Browse for Directory...
								</div>
								<div class="text-xs mt-1 opacity-70" style="color: var(--color-text-color);">
									Select any directory to use as a vault
								</div>
							</div>
						</div>
					</div>

					{#if vaults.length === 0}
						<div class="text-center py-8">
							<div class="text-gray-400">
								<Folder size={32} class="mx-auto mb-2" />
								<div class="text-sm font-medium mb-1">No vaults found</div>
								<div class="text-xs">Browse for a directory to create your first vault</div>
							</div>
						</div>
					{/if}
				</div>
			{/if}
		</div>

		<!-- Modal footer with shortcuts -->
		<div class="p-3 border-t border-gray-700" style="background-color: color-mix(in srgb, var(--color-panel-bg) 80%, black);">
			<div class="flex items-center justify-between text-xs text-gray-400">
				<div class="flex items-center gap-3">
					<span><kbd class="px-1 py-0.5 bg-gray-700 rounded text-xs">↑↓</kbd> Navigate</span>
					<span><kbd class="px-1 py-0.5 bg-gray-700 rounded text-xs">Enter</kbd> Select</span>
				</div>
				<span><kbd class="px-1 py-0.5 bg-gray-700 rounded text-xs">Esc</kbd> Close</span>
			</div>
		</div>
	</div>
{/if}

<style>
	/* Custom scrollbar for vault list */
	div::-webkit-scrollbar {
		width: 6px;
	}

	div::-webkit-scrollbar-track {
		background: transparent;
	}

	div::-webkit-scrollbar-thumb {
		background-color: rgba(156, 163, 175, 0.5);
		border-radius: 3px;
	}

	div::-webkit-scrollbar-thumb:hover {
		background-color: rgba(107, 114, 128, 0.7);
	}

	:global(.dark) div::-webkit-scrollbar-thumb {
		background-color: rgba(107, 114, 128, 0.5);
	}

	:global(.dark) div::-webkit-scrollbar-thumb:hover {
		background-color: rgba(156, 163, 175, 0.7);
	}

	/* Keyboard shortcuts styling */
	kbd {
		font-family: ui-monospace, SFMono-Regular, "SF Mono", Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
		font-size: 0.75rem;
		font-weight: 500;
	}
</style>
