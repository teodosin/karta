<!--
Runtime Node Wrapper - Unified Edition
Renders any node type using the unified registry in runtime mode
-->
<script lang="ts">
	import type { DataNode, ViewNode } from '$lib/types/types';
	import { unifiedNodeRegistry } from '../stores/UnifiedNodeRegistry';
	import { onMount } from 'svelte';

	export let dataNode: DataNode;
	export let viewNode: ViewNode;
	export let width: number;
	export let height: number;

	let nodeComponent: any = null;
	let isReady = false;

	onMount(async () => {
		// Ensure registry is initialized
		await unifiedNodeRegistry.initialize();
		
		// Get component for this node type in runtime mode
		const ComponentClass = unifiedNodeRegistry.getComponent(dataNode.ntype, 'runtime');
		if (ComponentClass) {
			nodeComponent = ComponentClass;
		}
		
		isReady = true;
	});
</script>

{#if isReady && nodeComponent}
	<svelte:component 
		this={nodeComponent} 
		{dataNode} 
		{viewNode} 
		mode="runtime"
	/>
{:else if isReady}
	<!-- Fallback for when component couldn't be loaded -->
	<div 
		class="w-full h-full p-2 bg-gray-200 border border-gray-400 rounded flex items-center justify-center"
		style="min-width: {width}px; min-height: {height}px;"
	>
		<div class="text-center text-gray-600">
			<div class="font-semibold">{dataNode.attributes?.name || 'Unknown Node'}</div>
			<div class="text-xs">Type: {dataNode.ntype}</div>
			<div class="text-xs">Runtime mode not supported</div>
		</div>
	</div>
{:else}
	<!-- Loading state -->
	<div 
		class="w-full h-full p-2 bg-gray-100 border border-gray-300 rounded flex items-center justify-center"
		style="min-width: {width}px; min-height: {height}px;"
	>
		<div class="text-gray-500 text-sm">Loading...</div>
	</div>
{/if}
