<script context="module" lang="ts">
	// MODULE SCRIPT (runs once)
	import type { SvelteComponent } from 'svelte';
	import type { TweenableNodeState } from '$lib/types/types';
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
	export const nodeTypeDef: Omit<NodeTypeDefinition, 'component'> = {
		ntype: 'root',
		getDefaultAttributes,
		getDefaultViewNodeState,
		displayName: 'Root',
		icon: undefined // Icon is imported/used in instance script, registry can access it via component if needed later
	};
</script>

<script lang="ts">
	// INSTANCE SCRIPT (runs for each component instance)
	import type { DataNode, ViewNode } from '$lib/types/types';
	import { currentContextId } from '$lib/karta/KartaStore';
	// Import BrainCog again for the template
	import { BrainCog } from 'lucide-svelte';

	export let dataNode: DataNode;
	export let viewNode: ViewNode;

	// Instance-specific logic here (if any)...
</script>
<!-- Root Node Appearance - Apply focus ring here -->
<div
	class={`
		w-full h-full rounded-full border-4 border-dashed border-orange-400 bg-orange-800
		flex items-center justify-center p-2 pointer-events-auto
		${dataNode.id === $currentContextId ? 'ring-4 ring-offset-2 ring-offset-gray-900 ring-orange-500' : ''}
	`}
	title={`Root Node: ${dataNode?.attributes?.name ?? dataNode.id}`}
>
	<!-- Use Lucide BrainCog icon -->
	<BrainCog class="h-1/2 w-1/2 text-orange-300 opacity-60 select-none pointer-events-none" strokeWidth={1.5} />
</div>

<style>
	/* Add any specific styles for the root node's internal content if needed */
	div {
		box-sizing: border-box; /* Ensure border is included in size */
	}
</style>