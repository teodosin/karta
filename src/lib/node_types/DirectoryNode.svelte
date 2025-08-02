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
	import { Folder } from 'lucide-svelte';

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
		icon: Folder as IconComponent,
		propertySchema: directoryNodePropertySchema
	};
</script>

<script lang="ts">
	// INSTANCE SCRIPT
	import type { DataNode, ViewNode } from '$lib/types/types';
	import { currentContextId, existingContextsMap, switchContext } from '$lib/karta/ContextStore';

	export let dataNode: DataNode;
	export let viewNode: ViewNode;
	$: hasContext = $existingContextsMap.has(viewNode.id);

	// Handle double-click to travel to directory's context
	function handleDoubleClick() {
		switchContext({ type: 'uuid', value: dataNode.id });
	}

	$: ringClasses = dataNode.id === $currentContextId
		? 'ring-4 ring-offset-2 ring-offset-gray-900 ring-[var(--color-focal-hl)]' // Focal highlight
		: hasContext
			? 'ring-2 ring-[var(--color-focal-hl)]' // Use border for context outline
			: ''; // No border/ring
</script>
<!-- Directory Node Appearance: Simple Rounded Rectangle - Apply focus ring here -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class={`
		w-full h-full rounded-md bg-panel-bg
		shadow-inner flex items-center justify-center pointer-events-auto
		${ringClasses}
	`}
	on:dblclick={handleDoubleClick}
>
	<Folder class="text-[var(--color-text)] opacity-40" size={40} fill="currentColor" />
</div>

<style>
	div {
		box-sizing: border-box;
	}
</style>