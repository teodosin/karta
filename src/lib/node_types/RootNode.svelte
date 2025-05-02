<!--
// --- Karta Runtime Component ---
// This file is planned for inclusion in the MIT-licensed `karta_runtime` package.
// It defines the rendering and behavior for the 'root' node type.
// Play Mode interactions should be handled based on node attributes.
-->
<script context="module" lang="ts">
	// MODULE SCRIPT (runs once)
	import type { SvelteComponent } from 'svelte';
	import type { TweenableNodeState, PropertyDefinition } from '$lib/types/types'; // Import PropertyDefinition
	// Import Omit<NodeTypeDefinition, 'component'> if needed, or just define structure
	import type { NodeTypeDefinition, IconComponent } from './types';
	// BrainCog is imported in instance script

	function getDefaultAttributes(baseName = 'Root'): Record<string, any> {
		return { name: baseName };
	}

	function getDefaultViewNodeState(): Omit<TweenableNodeState, 'x' | 'y'> {
		return { width: 50, height: 50, scale: 1, rotation: 0 };
	}

	// Export the static definition from the module script
	// Note: We omit 'component' here as it's implicitly the default export
	const rootNodePropertySchema: PropertyDefinition[] = []; // No type-specific properties for Root

	export const nodeTypeDef: Omit<NodeTypeDefinition, 'component'> = {
		ntype: 'root',
		getDefaultAttributes,
		getDefaultViewNodeState,
		displayName: 'Root',
		icon: undefined, // Icon is imported/used in instance script, registry can access it via component if needed later
		propertySchema: rootNodePropertySchema
	};
</script>

<script lang="ts">
	// INSTANCE SCRIPT (runs for each component instance)
	import type { DataNode, ViewNode } from '$lib/types/types';
	import { currentContextId } from '$lib/karta/ContextStore'; // Corrected import
	// Import BrainCog again for the template
	import { BrainCog } from 'lucide-svelte';

	export let dataNode: DataNode;
	export let viewNode: ViewNode;

	// Instance-specific logic here (if any)...
</script>
<!-- Root Node Appearance - Apply focus ring here -->
<div
	class={`
		w-full h-full rounded-full border-2 border-dashed border-orange-400 bg-wine
		flex items-center justify-center p-2 pointer-events-auto
		${dataNode.id === $currentContextId ? 'ring-4 ring-offset-2 ring-offset-gray-900 ring-orange-500' : ''}
	`}
	title={`Root Node: ${dataNode?.attributes?.name ?? dataNode.id}`}
>
	<!-- Use Lucide BrainCog icon -->
	<BrainCog class="select-none pointer-events-none" strokeWidth={1.5} size={50} color="orange" />
</div>

<style>
	/* Add any specific styles for the root node's internal content if needed */
	div {
		box-sizing: border-box; /* Ensure border is included in size */
	}
</style>