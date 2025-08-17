import { invoke } from '@tauri-apps/api/core';
import { API_BASE } from '$lib/apiBase';

export interface VaultInfo {
    path: string;
    exists: boolean;
    has_karta_dir: boolean;
}

export interface ServerManager {
    startServer(vaultPath: string): Promise<void>;
    stopServer(): Promise<void>;
    checkServerStatus(): Promise<boolean>;
    pollForServerReady(maxAttempts?: number, intervalMs?: number): Promise<void>;
    getAvailableVaults(): Promise<VaultInfo[]>;
    selectVaultDirectory(): Promise<string | null>;
    addVaultToConfig(vaultPath: string): Promise<void>;
}

export const serverManager: ServerManager = {
    async startServer(vaultPath: string): Promise<void> {
        return await invoke('start_server', { vaultPath });
    },

    async stopServer(): Promise<void> {
        return await invoke('stop_server');
    },

    async checkServerStatus(): Promise<boolean> {
        // Prefer browser-side fetch to avoid blocking native event loop on Linux
        const controller = new AbortController();
        const timeout = setTimeout(() => controller.abort(), 1000);
        try {
            const res = await fetch(`${API_BASE}/`, { method: 'GET', signal: controller.signal, cache: 'no-store' });
            return res.ok;
        } catch {
            return false;
        } finally {
            clearTimeout(timeout);
        }
    },

    async pollForServerReady(maxAttempts = 30, intervalMs = 500): Promise<void> {
        for (let i = 0; i < maxAttempts; i++) {
            const isReady = await this.checkServerStatus();
            if (isReady) {
                return;
            }
            
            // Wait before next attempt
            await new Promise(resolve => setTimeout(resolve, intervalMs));
        }
        
        throw new Error(`Server failed to start within ${(maxAttempts * intervalMs) / 1000} seconds`);
    },

    async getAvailableVaults(): Promise<VaultInfo[]> {
        return await invoke('get_available_vaults');
    },

    async selectVaultDirectory(): Promise<string | null> {
        return await invoke('select_vault_directory');
    },

    async addVaultToConfig(vaultPath: string): Promise<void> {
        return await invoke('add_vault_to_config', { vaultPath });
    }
};
