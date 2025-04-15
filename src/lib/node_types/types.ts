// --- Karta Runtime Component ---
// This file is planned for inclusion in the MIT-licensed `karta_runtime` package.
// It defines types related to node type definitions and their static properties.
// Essential for both editor and runtime.

import type { SvelteComponent } from 'svelte';
import type { DataNode, ViewNode, TweenableNodeState, PropertyDefinition } from '$lib/types/types'; // Import base types and PropertyDefinition

// Define the expected props for any node type component rendered by NodeWrapper
export interface NodeTypeProps {
	dataNode: DataNode;
	viewNode: ViewNode;
}

// Define a specific type for our node components using Svelte 5's component type
export type NodeTypeComponent = typeof SvelteComponent<NodeTypeProps>;

// Define a type for simple icon components (optional)
// Using 'any' for props as icon props can vary widely (size, strokeWidth, class, etc.)
export type IconComponent = typeof SvelteComponent<any>;


// Defines the contract for a node type module
export interface NodeTypeDefinition {
	// The unique string identifier for this node type (matches DataNode.ntype)
	ntype: string;

	// The Svelte component for rendering the node's content
	// component: NodeTypeComponent; // Component is the default export of the .svelte file

	// Function to get the default data attributes for a new node of this type
	// Takes an optional baseName suggestion
	getDefaultAttributes: (baseName?: string) => Record<string, any>;

	// Function to get the default intrinsic visual properties (size, initial scale/rotation)
	// Excludes position (x, y) which is determined at creation time.
	getDefaultViewNodeState: () => Omit<TweenableNodeState, 'x' | 'y'>;

	// Optional properties for future use (e.g., in menus)
	displayName?: string; // User-friendly name for UI
	icon?: IconComponent; // e.g., a Lucide icon component

	// Optional schema defining editable properties for this node type
	propertySchema?: PropertyDefinition[];
}