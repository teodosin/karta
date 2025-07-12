<!--
// --- Karta Runtime Component ---
// This file is planned for inclusion in the MIT-licensed `karta_runtime` package.
// It defines the rendering and behavior for the 'core/fs/dir' node type.
// Play Mode interactions should be handled based on node attributes.
-->
<script context="module" lang="ts">
	// MODULE SCRIPT
	import type { TweenableNodeState, PropertyDefinition } from '$lib/types/types';
	import type { NodeTypeDefinition, IconComponent } from './types';
	// Optional: import icon like Folder from 'lucide-svelte'; // Example for DirectoryNode

	function getDefaultAttributes(baseName = 'Directory'): Record<string, any> { // Changed baseName
		return {
			name: baseName,
			view_isNameVisible: true // Generic view default
		};
	}

	function getDefaultViewNodeState(): Omit<TweenableNodeState, 'x' | 'y'> {
		return { width: 100, height: 100, scale: 1, rotation: 0 };
	}

	const directoryNodePropertySchema: PropertyDefinition[] = []; // No type-specific properties for now

	export const nodeTypeDef: Omit<NodeTypeDefinition, 'component'> = {
		ntype: 'core/fs/dir', // Changed ntype
		getDefaultAttributes,
		getDefaultViewNodeState,
		displayName: 'Directory', // Changed displayName
		// icon: Folder as IconComponent // Example
		propertySchema: directoryNodePropertySchema
	};
</script>

<script lang="ts">
	// INSTANCE SCRIPT
	import type { DataNode, ViewNode } from '$lib/types/types';
	import { currentContextId, availableContextsMap } from '$lib/karta/ContextStore';

	export let dataNode: DataNode;
	export let viewNode: ViewNode;
	$: hasContext = $availableContextsMap.has(viewNode.id);

	// Instance logic...

	$: ringClasses = dataNode.id === $currentContextId
		? 'ring-4 ring-offset-2 ring-offset-gray-900 ring-[var(--color-focal-hl)]' // Focal highlight
		: hasContext
			? 'ring-2 ring-[var(--color-focal-hl)]' // Use border for context outline
			: ''; // No border/ring
</script>
<!-- Directory Node Appearance: Simple Rounded Rectangle - Apply focus ring here -->
<div
	class={`
		w-full h-full rounded-md bg-panel-bg
		shadow-inner flex items-center justify-center pointer-events-auto
		${ringClasses}
	`}
>
	<!-- Optional: Could add a subtle icon or pattern, e.g., a folder icon -->
	<!-- <span class="text-white opacity-50 text-xs">D</span> -->
</div>

<style>
	div {
		box-sizing: border-box;
	}
</style>