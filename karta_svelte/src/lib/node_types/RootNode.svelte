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
		return {
			name: baseName,
			view_isNameVisible: false // Generic view default, root name is usually hidden
		};
	}

	function getDefaultViewNodeState(): Omit<TweenableNodeState, 'x' | 'y'> {
		return { width: 50, height: 50, scale: 1, rotation: 0 };
	}

	// Export the static definition from the module script
	// Note: We omit 'component' here as it's implicitly the default export
	const rootNodePropertySchema: PropertyDefinition[] = []; // No type-specific properties for Root

	export const nodeTypeDef: Omit<NodeTypeDefinition, 'component'> = {
		ntype: 'core/root',
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
	import { currentContextId, existingContextsMap } from '$lib/karta/ContextStore'; // Import existingContextsMap
	import { settings } from '$lib/karta/SettingsStore';
	// Import BrainCog again for the template
	import { BrainCog } from 'lucide-svelte';

	export let dataNode: DataNode;
	export let viewNode: ViewNode;
	// Check if context exists for this node using the new map
	$: hasContext = $existingContextsMap.has(viewNode.id);

	// Instance-specific logic here (if any)...

	// Determine ring classes based on focal state and context existence
	$: ringClasses = dataNode.id === $currentContextId
		? 'ring-4 ring-offset-2 ring-offset-gray-900 ring-[var(--color-focal-hl)]' // Focal highlight
		: hasContext
			? 'ring-2 ring-[var(--color-focal-hl)]' // Use border for context outline
			: ''; // No border/ring
</script>
<!-- Root Node Appearance - Apply focus ring here -->
<div
	class={`
		w-full h-full rounded-full border-2 border-dashed bg-panel-bg
		flex items-center justify-center p-2 pointer-events-auto
		${ringClasses}
	`}
	style="border-color: var(--color-focal-hl);"
>
	<!-- Use Lucide BrainCog icon -->
	<BrainCog class="select-none pointer-events-none" strokeWidth={1.5} size={50} color="var(--color-focal-hl)" />
</div>

<style>
	/* Add any specific styles for the root node's internal content if needed */
	div {
		box-sizing: border-box; /* Ensure border is included in size */
	}
</style>