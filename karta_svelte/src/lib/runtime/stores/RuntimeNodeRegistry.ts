/**
 * RuntimeNodeRegistry - Runtime-compatible node type registry
 * 
 * This provides a simplified registry for the runtime environment that can:
 * 1. Auto-discover runtime node components from a folder using Vite's glob import
 * 2. Allow plugins to register custom runtime node components
 * 3. Provide fallback components for unknown types
 */

import type { SvelteComponent } from 'svelte';
import type { DataNode, ViewNode } from '../../types/types';

// Runtime-specific component interface
export interface RuntimeNodeComponent {
    // Any Svelte component constructor
    new (...args: any[]): SvelteComponent;
}

// Props that every runtime node component will receive
export interface RuntimeNodeProps {
    dataNode: DataNode;
    viewNode: ViewNode;
    width: number;
    height: number;
}

// Runtime node definition (exported from each runtime node component)
export interface RuntimeNodeDefinition {
    ntype: string;
    displayName?: string;
    fallback?: boolean; // Mark as fallback component
}

// Registry entry for runtime components
interface RuntimeRegistryEntry {
    component: RuntimeNodeComponent;
    definition: RuntimeNodeDefinition;
}

class RuntimeNodeRegistry {
    private components = new Map<string, RuntimeRegistryEntry>();
    private fallbackComponent: RuntimeNodeComponent | null = null;
    private initialized = false;

    /**
     * Auto-discover and register runtime node components from folder
     */
    async initialize(): Promise<void> {
        if (this.initialized) return;

        // Use Vite's glob import to load all runtime node components
        const modules = import.meta.glob<any>('../components/runtime-nodes/*.svelte', { 
            eager: true 
        });

        console.log('[RuntimeNodeRegistry] Discovered modules:', Object.keys(modules));

        for (const path in modules) {
            const module = modules[path];
            
            // Access the named export 'runtimeNodeDef' and default export (component)
            const definition = module.runtimeNodeDef as RuntimeNodeDefinition;
            const component = module.default as RuntimeNodeComponent;

            if (definition && definition.ntype && component) {
                this.components.set(definition.ntype, { component, definition });
                
                // Set as fallback if marked
                if (definition.fallback) {
                    this.fallbackComponent = component;
                }
                
                console.log(`[RuntimeNodeRegistry] Registered: ${definition.ntype} (${definition.displayName || 'unnamed'})`);
            } else {
                let warnings = [];
                if (!definition) warnings.push("runtimeNodeDef export missing");
                else if (!definition.ntype) warnings.push("ntype missing in runtimeNodeDef");
                if (!component) warnings.push("default export (component) missing");
                console.warn(`[RuntimeNodeRegistry] Failed to load from ${path}. Issues: ${warnings.join(', ')}`);
            }
        }

        this.initialized = true;
        console.log('[RuntimeNodeRegistry] Initialization complete. Registered types:', this.getRegisteredTypes());
    }

    /**
     * Register a runtime component for a specific node type (for plugins)
     */
    register(ntype: string, component: RuntimeNodeComponent, displayName?: string): void {
        const definition: RuntimeNodeDefinition = { ntype, displayName };
        this.components.set(ntype, { component, definition });
        console.log(`[RuntimeNodeRegistry] Manually registered: ${ntype}`);
    }

    /**
     * Set the fallback component for unknown node types
     */
    setFallback(component: RuntimeNodeComponent): void {
        this.fallbackComponent = component;
    }

    /**
     * Get the component for a specific node type
     */
    getComponent(ntype: string): RuntimeNodeComponent | null {
        // Ensure initialization
        if (!this.initialized) {
            console.warn('[RuntimeNodeRegistry] Not initialized yet, auto-initializing...');
            this.initialize();
        }

        const entry = this.components.get(ntype);
        if (entry) {
            return entry.component;
        }
        
        // Return fallback if available
        return this.fallbackComponent;
    }

    /**
     * Get all registered node types
     */
    getRegisteredTypes(): string[] {
        return Array.from(this.components.keys());
    }

    /**
     * Check if a node type is registered
     */
    hasType(ntype: string): boolean {
        return this.components.has(ntype);
    }

    /**
     * Get definition for a node type
     */
    getDefinition(ntype: string): RuntimeNodeDefinition | undefined {
        return this.components.get(ntype)?.definition;
    }

    /**
     * Clear all registrations (useful for testing)
     */
    clear(): void {
        this.components.clear();
        this.fallbackComponent = null;
        this.initialized = false;
    }
}

// Export singleton instance
export const runtimeNodeRegistry = new RuntimeNodeRegistry();
