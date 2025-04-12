<!--
// --- Karta Runtime Component ---
// This file is planned for inclusion in the MIT-licensed `karta_runtime` package.
// It defines the rendering and behavior for the 'image' node type.
// Play Mode interactions should be handled based on node attributes.
-->
<script context="module" lang="ts">
	// MODULE SCRIPT
	import type { TweenableNodeState } from '$lib/types/types';
	import type { NodeTypeDefinition, IconComponent } from './types';
	// Optional: import { Image } from 'lucide-svelte';

	function getDefaultAttributes(baseName = 'Image'): Record<string, any> {
		return { name: baseName, src: '', alt: '' }; // Placeholder src
	}

	function getDefaultViewNodeState(): Omit<TweenableNodeState, 'x' | 'y'> {
		return { width: 100, height: 100, scale: 1, rotation: 0 };
	}

	export const nodeTypeDef: Omit<NodeTypeDefinition, 'component'> = {
		ntype: 'image',
		getDefaultAttributes,
		getDefaultViewNodeState,
		displayName: 'Image'
		// icon: Image as IconComponent // Example
	};
</script>

<script lang="ts">
	// INSTANCE SCRIPT
	import type { DataNode, ViewNode } from '$lib/types/types';
	import { currentContextId } from '$lib/karta/KartaStore';

	export let dataNode: DataNode;
	export let viewNode: ViewNode;

	// Type assertion for attributes
	$: attributes = dataNode.attributes as { src?: string; alt?: string; name: string };
	$: imgSrc = attributes?.src || ''; // Use src attribute or empty string
	$: altText = attributes?.alt || `Image for ${attributes?.name ?? dataNode.id}`;

	// Placeholder image (e.g., SVG or data URI) if src is empty
	const placeholderSvg = `
		<svg xmlns="http://www.w3.org/2000/svg" class="w-full h-full text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1">
			<path stroke-linecap="round" stroke-linejoin="round" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
		</svg>
	`;
	const placeholderDataUri = `data:image/svg+xml;utf8,${encodeURIComponent(placeholderSvg)}`;

	// Instance logic...
</script>
<div
	class={`
		w-full h-full bg-gray-700 flex items-center justify-center overflow-hidden pointer-events-auto
		${dataNode.id === $currentContextId ? 'ring-4 ring-offset-2 ring-offset-gray-900 ring-orange-500 rounded' : 'rounded shadow-md'}
	`}
	title={`Image Node: ${attributes?.name ?? dataNode.id}`}
>
	{#if imgSrc}
		<img
			src={imgSrc}
			alt={altText}
			class="object-contain w-full h-full pointer-events-none"
			loading="lazy"
			on:error={(e) => {
				// Prevent infinite loop if placeholder also fails
				if (e.currentTarget instanceof HTMLImageElement && e.currentTarget.src !== placeholderDataUri) {
					e.currentTarget.src = placeholderDataUri;
				}
			}}
		/>
	{:else}
		<!-- Display placeholder SVG directly -->
		<div class="pointer-events-none w-full h-full flex items-center justify-center">{@html placeholderSvg}</div>
	{/if}
</div>

<style>
	div {
		box-sizing: border-box;
	}
	img {
		display: block; /* Remove extra space below image */
		max-width: 100%;
		max-height: 100%;
	}
</style>