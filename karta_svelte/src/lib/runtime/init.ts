/**
 * Runtime Node Registry Initialization
 * 
 * This file initializes the runtime node registry with auto-discovery.
 * It automatically loads all components from the runtime-nodes folder.
 */

import { runtimeNodeRegistry } from './stores/RuntimeNodeRegistry';

// Initialize the registry with auto-discovery
export async function initializeRuntimeNodeTypes(): Promise<void> {
    try {
        await runtimeNodeRegistry.initialize();
        console.log('[Runtime] Auto-discovery complete. Available types:', runtimeNodeRegistry.getRegisteredTypes());
    } catch (error) {
        console.error('[Runtime] Failed to initialize node registry:', error);
    }
}

// Auto-initialize when module is imported (but allow manual initialization too)
if (typeof window !== 'undefined') {
    // Only auto-init in browser environment
    initializeRuntimeNodeTypes();
}
