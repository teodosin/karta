<!--
// --- Karta Runtime Component ---
// This file is planned for inclusion in the MIT-licensed `karta_runtime` package.
// It defines the rendering and behavior for the 'core/fs/file' node type.
// Play Mode interactions should be handled based on node attributes.
-->
<script context="module" lang="ts">
	// MODULE SCRIPT
	import type { TweenableNodeState, PropertyDefinition } from '$lib/types/types'; // Import PropertyDefinition
	import type { NodeTypeDefinition, IconComponent } from './types';
	// Optional: import icon like FileText from 'lucide-svelte'; // Example for FileNode

	function getDefaultAttributes(baseName = 'File'): Record<string, any> { // Changed baseName
		return {
			name: baseName,
			view_isNameVisible: true // Generic view default
		};
	}

	function getDefaultViewNodeState(): Omit<TweenableNodeState, 'x' | 'y'> {
		return { width: 100, height: 100, scale: 1, rotation: 0 };
	}

	const fileNodePropertySchema: PropertyDefinition[] = []; // No type-specific properties for now

	export const nodeTypeDef: Omit<NodeTypeDefinition, 'component'> = {
		ntype: 'core/fs/file', // Changed ntype
		getDefaultAttributes,
		getDefaultViewNodeState,
		displayName: 'File', // Changed displayName
		// icon: FileText as IconComponent // Example
		propertySchema: fileNodePropertySchema
	};
</script>

<script lang="ts">
	// INSTANCE SCRIPT
	import type { DataNode, ViewNode } from '$lib/types/types';
	import { currentContextId, existingContextsMap } from '$lib/karta/ContextStore'; // Import existingContextsMap

	export let dataNode: DataNode;
	export let viewNode: ViewNode;
	// Check if context exists for this node using the new map
	$: hasContext = $existingContextsMap.has(viewNode.id);

	// Extract file extension from the node name
	$: fileExtension = (() => {
		const name = dataNode.attributes.name || '';
		const lastDotIndex = name.lastIndexOf('.');
		if (lastDotIndex > 0 && lastDotIndex < name.length - 1) {
			return name.substring(lastDotIndex + 1).toUpperCase();
		}
		return 'FILE';
	})();

	// Instance logic...

	// Determine ring classes based on focal state and context existence
	$: ringClasses = dataNode.id === $currentContextId
		? 'ring-4 ring-offset-2 ring-offset-gray-900 ring-[var(--color-focal-hl)]' // Focal highlight
		: hasContext
			? 'ring-2 ring-[var(--color-focal-hl)]' // Use border for context outline
			: ''; // No border/ring
</script>
<!-- File Node Appearance: Simple Rounded Rectangle - Apply focus ring here -->
<div
	class={`
		w-full h-full rounded-[1rem] bg-panel-bg mb-4 p-2
		shadow-inner flex items-center justify-center pointer-events-auto
		${ringClasses}
	`}
>
	<div class="h-full w-full bg-viewport-bg/50 rounded-[0.5rem] flex items-center justify-center">
		<span class="text-[var(--color-text)] opacity-70 text-xs font-medium text-center px-1 leading-tight">
			{fileExtension}
		</span>
	</div>
</div>

<style>
	div {
		box-sizing: border-box;
	}
</style>