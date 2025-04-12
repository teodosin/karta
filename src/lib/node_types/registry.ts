// --- Karta Runtime Component ---
// This file is planned for inclusion in the MIT-licensed `karta_runtime` package.
// It dynamically loads and registers available node type components and definitions.
// This is essential for both the editor and the runtime.

import type { SvelteComponent } from 'svelte';
import type { NodeTypeDefinition, NodeTypeComponent, IconComponent } from './types'; // Include IconComponent
import type { TweenableNodeState } from '$lib/types/types';

// Use Vite's glob import to eagerly load all Svelte component modules in this directory
// Import the entire module to access both default (component) and named (nodeTypeDef) exports
const modules = import.meta.glob<any>('./*.svelte', { // Use 'any' for module type
	eager: true
});

// Define the structure for the registry map
// Key: ntype (string), Value: Combined object with definition and component
interface RegistryEntry {
	// Store definition without component ref, as component is handled separately
	definition: Omit<NodeTypeDefinition, 'component'>;
	component: NodeTypeComponent;
}
export const nodeTypeRegistry: Record<string, RegistryEntry> = {};

// Populate the registry dynamically
for (const path in modules) {
	const module = modules[path];
	// Access the named export 'nodeTypeDef'
	const definition = module.nodeTypeDef as Omit<NodeTypeDefinition, 'component'>;
	// Access the default export (the component)
	const component = module.default as NodeTypeComponent;

	if (definition && definition.ntype && component) {
		nodeTypeRegistry[definition.ntype] = {
			definition: definition,
			component: component
		};
		console.log(`[Registry] Registered node type: ${definition.ntype}`);
	} else {
		// Add more specific warnings
		let warnings = [];
		if (!module) warnings.push("module not loaded");
		if (!definition) warnings.push("nodeTypeDef export missing");
		else if (!definition.ntype) warnings.push("ntype missing in nodeTypeDef");
		if (!component) warnings.push("default export (component) missing");
		console.warn(`[Registry] Failed to load node type from path: ${path}. Issues: ${warnings.join(', ')}.`);
	}
}

// --- Registry Helper Functions ---

// Get the full definition object (excluding component) for a type
export function getNodeTypeDef(ntype: string): Omit<NodeTypeDefinition, 'component'> | undefined {
	return nodeTypeRegistry[ntype]?.definition;
}

// Function to get the component constructor for a type
export function getNodeComponent(ntype: string): NodeTypeComponent | undefined {
	const entry = nodeTypeRegistry[ntype];
	// Fallback to generic component if type not found or component missing
	return entry?.component || nodeTypeRegistry['generic']?.component;
}

// Function to get default attributes for a type
export function getDefaultAttributesForType(ntype: string): Record<string, any> {
	const definition = nodeTypeRegistry[ntype]?.definition;
	if (definition?.getDefaultAttributes) {
		// Generate a base name suggestion
		const baseName = definition.displayName || ntype.charAt(0).toUpperCase() + ntype.slice(1);
		return definition.getDefaultAttributes(baseName);
	}
	// Fallback if function missing
	console.warn(`getDefaultAttributes function missing for ntype: ${ntype}`);
	return { name: `New ${ntype}` };
}

// Function to get default view state for a type
export function getDefaultViewNodeStateForType(ntype: string): Omit<TweenableNodeState, 'x' | 'y'> {
	const definition = nodeTypeRegistry[ntype]?.definition;
	if (definition?.getDefaultViewNodeState) {
		return definition.getDefaultViewNodeState();
	}
	// Fallback to generic defaults
	console.warn(`getDefaultViewNodeState function missing for ntype: ${ntype}, using fallback.`);
	return { width: 100, height: 100, scale: 1, rotation: 0 };
}

// Function to get a list of available types for UI (excluding root)
export function getAvailableNodeTypesForMenu(): { ntype: string; displayName: string; icon?: IconComponent }[] {
    return Object.values(nodeTypeRegistry)
        .filter(entry => entry.definition.ntype !== 'root') // Exclude root type
        .map(entry => ({
            ntype: entry.definition.ntype,
            displayName: entry.definition.displayName || entry.definition.ntype,
            icon: entry.definition.icon // Include icon if defined
        }))
        .sort((a, b) => a.displayName.localeCompare(b.displayName)); // Sort alphabetically
}