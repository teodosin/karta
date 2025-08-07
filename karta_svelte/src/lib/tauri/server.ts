import { invoke } from '@tauri-apps/api/core';

export interface VaultInfo {
    path: string;
    exists: boolean;
    has_karta_dir: boolean;
}

export interface ServerManager {
    startServer(vaultPath: string): Promise<void>;
    stopServer(): Promise<void>;
    checkServerStatus(): Promise<boolean>;
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
        return await invoke('check_server_status');
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
