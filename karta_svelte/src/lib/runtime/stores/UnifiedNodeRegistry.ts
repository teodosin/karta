/**
 * UnifiedNodeRegistry - Unified node type registry for both editor and runtime
 * 
 * This provides a unified registry that can:
 * 1. Auto-discover unified node types from the node_types folder
 * 2. Render the same components in both editor and runtime modes
 * 3. Allow plugins to register custom node types
 * 4. Provide fallback components for unknown types
 */

import type { SvelteComponent } from 'svelte';
import type { DataNode, ViewNode } from '../../types/types';

// Unified component interface - same component for editor and runtime
export interface UnifiedNodeComponent {
    // Any Svelte component constructor
    new (...args: any[]): SvelteComponent;
}

// Props that every unified node component receives
export interface UnifiedNodeProps {
    dataNode: DataNode;
    viewNode: ViewNode;
    mode: 'editor' | 'runtime';
    // Additional props for editor mode
    width?: number;
    height?: number;
}

// Unified node definition (exported from each node type component)
export interface UnifiedNodeDefinition {
    ntype: string;
    displayName?: string;
    supportsRuntime?: boolean; // Whether this node type supports runtime mode
    fallback?: boolean; // Mark as fallback component
    getDefaultAttributes?: (baseName?: string) => Record<string, any>;
    getDefaultViewNodeState?: () => any;
    propertySchema?: any[];
}

// Registry entry for unified components
interface UnifiedRegistryEntry {
    component: UnifiedNodeComponent;
    definition: UnifiedNodeDefinition;
}

class UnifiedNodeRegistry {
    private components = new Map<string, UnifiedRegistryEntry>();
    private fallbackComponent: UnifiedNodeComponent | null = null;
    private initialized = false;

    /**
     * Auto-discover and register unified node types from node_types folder
     */
    async initialize(): Promise<void> {
        if (this.initialized) return;

        // Use Vite's glob import to load all node type components
        const modules = import.meta.glob<any>('../../node_types/*.svelte', { 
            eager: true 
        });

        console.log('[UnifiedNodeRegistry] Discovered modules:', Object.keys(modules));

        for (const path in modules) {
            const module = modules[path];
            
            // Access the named export 'nodeTypeDef' and default export (component)
            const definition = module.nodeTypeDef as UnifiedNodeDefinition;
            const component = module.default as UnifiedNodeComponent;

            if (definition && definition.ntype && component) {
                this.components.set(definition.ntype, { component, definition });
                
                // Set as fallback if marked
                if (definition.fallback) {
                    this.fallbackComponent = component;
                }
                
                console.log(`[UnifiedNodeRegistry] Registered: ${definition.ntype} (${definition.displayName || 'unnamed'}) - Runtime support: ${definition.supportsRuntime || false}`);
            } else {
                let warnings = [];
                if (!definition) warnings.push("nodeTypeDef export missing");
                else if (!definition.ntype) warnings.push("ntype missing in nodeTypeDef");
                if (!component) warnings.push("default export (component) missing");
                console.warn(`[UnifiedNodeRegistry] Failed to load from ${path}. Issues: ${warnings.join(', ')}`);
            }
        }

        this.initialized = true;
        console.log('[UnifiedNodeRegistry] Initialization complete. Registered types:', this.getRegisteredTypes());
    }

    /**
     * Register a unified component for a specific node type (for plugins)
     */
    register(ntype: string, component: UnifiedNodeComponent, definition: Partial<UnifiedNodeDefinition> = {}): void {
        const fullDefinition: UnifiedNodeDefinition = { 
            ntype, 
            supportsRuntime: true,
            ...definition 
        };
        this.components.set(ntype, { component, definition: fullDefinition });
        console.log(`[UnifiedNodeRegistry] Manually registered: ${ntype}`);
    }

    /**
     * Set the fallback component for unknown node types
     */
    setFallback(component: UnifiedNodeComponent): void {
        this.fallbackComponent = component;
        console.log('[UnifiedNodeRegistry] Fallback component set');
    }

    /**
     * Get component for a node type in specified mode
     */
    getComponent(ntype: string, mode: 'editor' | 'runtime' = 'editor'): UnifiedNodeComponent | null {
        const entry = this.components.get(ntype);
        
        if (entry) {
            // Check if component supports runtime mode when requested
            if (mode === 'runtime' && !entry.definition.supportsRuntime) {
                console.warn(`[UnifiedNodeRegistry] Node type ${ntype} does not support runtime mode, using fallback`);
                return this.fallbackComponent;
            }
            return entry.component;
        }

        console.warn(`[UnifiedNodeRegistry] Unknown node type: ${ntype}, using fallback`);
        return this.fallbackComponent;
    }

    /**
     * Get definition for a node type
     */
    getDefinition(ntype: string): UnifiedNodeDefinition | null {
        const entry = this.components.get(ntype);
        return entry ? entry.definition : null;
    }

    /**
     * Check if registry has been initialized
     */
    isInitialized(): boolean {
        return this.initialized;
    }

    /**
     * Get all registered node types
     */
    getRegisteredTypes(): string[] {
        return Array.from(this.components.keys());
    }

    /**
     * Get all registered components with runtime support
     */
    getRuntimeSupportedTypes(): string[] {
        return Array.from(this.components.entries())
            .filter(([_, entry]) => entry.definition.supportsRuntime)
            .map(([ntype, _]) => ntype);
    }

    /**
     * Check if a node type supports runtime mode
     */
    supportsRuntime(ntype: string): boolean {
        const entry = this.components.get(ntype);
        return entry ? (entry.definition.supportsRuntime || false) : false;
    }

    /**
     * Get component entries for debugging
     */
    getAll(): Map<string, UnifiedRegistryEntry> {
        return new Map(this.components);
    }
}

// Export singleton instance
export const unifiedNodeRegistry = new UnifiedNodeRegistry();

// Convenience function to initialize registry
export async function initializeUnifiedNodeRegistry(): Promise<void> {
    await unifiedNodeRegistry.initialize();
}
