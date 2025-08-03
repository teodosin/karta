import { writable } from 'svelte/store';

export const vaultName = writable<string | null>(null);

export async function initializeVault() {

  try {
    const response = await fetch('http://localhost:7370/');

    if (!response.ok) {
      throw new Error('Failed to fetch vault info');
    }

    const data = await response.json();
    vaultName.set(data.vault_name);
    
  } catch (error) {
    console.error('[VaultStore] Error initializing vault:', error);
    vaultName.set(null);
  }
}