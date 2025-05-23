<!--
// --- Karta Runtime Component ---
// This file is planned for inclusion in the MIT-licensed `karta_runtime` package.
// It defines the rendering and behavior for the 'generic' node type.
// Play Mode interactions should be handled based on node attributes.
-->
<script context="module" lang="ts">
	// MODULE SCRIPT
	import type { TweenableNodeState, PropertyDefinition } from '$lib/types/types'; // Import PropertyDefinition
	import type { NodeTypeDefinition, IconComponent } from './types';
	// Optional: import icon like Circle from 'lucide-svelte';

	function getDefaultAttributes(baseName = 'Node'): Record<string, any> {
		return {
			name: baseName,
			view_isNameVisible: true // Generic view default
		};
	}

	function getDefaultViewNodeState(): Omit<TweenableNodeState, 'x' | 'y'> {
		return { width: 100, height: 100, scale: 1, rotation: 0 };
	}

	const genericNodePropertySchema: PropertyDefinition[] = []; // No type-specific properties

	export const nodeTypeDef: Omit<NodeTypeDefinition, 'component'> = {
		ntype: 'Node',
		getDefaultAttributes,
		getDefaultViewNodeState,
		displayName: 'Node',
		// icon: Circle as IconComponent // Example
		propertySchema: genericNodePropertySchema
	};
</script>

<script lang="ts">
	// INSTANCE SCRIPT
	import type { DataNode, ViewNode } from '$lib/types/types';
	import { currentContextId, availableContextsMap } from '$lib/karta/ContextStore'; // Import availableContextsMap

	export let dataNode: DataNode;
	export let viewNode: ViewNode;
	// Check if context exists for this node using the new map
	$: hasContext = $availableContextsMap.has(viewNode.id);

	// Instance logic...

	// Determine ring classes based on focal state and context existence
	$: ringClasses = dataNode.id === $currentContextId
		? 'ring-4 ring-offset-2 ring-offset-gray-900 ring-orange-500' // Focal highlight
		: hasContext
			? 'ring-2 ring-orange-500/50' // Use border for context outline
			: ''; // No border/ring
</script>
<!-- Generic Node Appearance: Simple Circle - Apply focus ring here -->
<div
	class={`
		w-full h-full rounded-full bg-wine
		shadow-inner flex items-center justify-center pointer-events-auto
		${ringClasses}
	`}
	title={`Generic Node: ${dataNode?.attributes?.name ?? dataNode.id}`}
>
	<!-- Optional: Could add a subtle icon or pattern -->
	<!-- <span class="text-white opacity-50 text-xs">G</span> -->
</div>

<style>
	div {
		box-sizing: border-box;
	}
</style>