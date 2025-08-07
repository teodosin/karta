/**
 * Unified Runtime Initialization
 * 
 * Initialize the unified node registry for both editor and runtime environments
 */

import { initializeUnifiedNodeRegistry } from './stores/UnifiedNodeRegistry';

/**
 * Initialize unified node types
 * This should be called once at application startup
 */
export async function initializeUnifiedNodeTypes(): Promise<void> {
    console.log('[Unified Runtime] Initializing unified node types...');
    await initializeUnifiedNodeRegistry();
    console.log('[Unified Runtime] Unified node types initialized successfully');
}

// Auto-initialize when module is imported (for convenience)
if (typeof window !== 'undefined') {
    // Only auto-initialize in browser environment
    initializeUnifiedNodeTypes().catch(console.error);
}
