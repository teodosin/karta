<!--
// --- Karta Runtime Component ---
// This file is planned for inclusion in the MIT-licensed `karta_runtime` package.
// It defines the rendering and behavior for the 'image' node type.
// Play Mode interactions should be handled based on node attributes.
-->
<script context="module" lang="ts">
	// MODULE SCRIPT
	import type { TweenableNodeState, PropertyDefinition } from '$lib/types/types'; // Import PropertyDefinition
	import type { NodeTypeDefinition, IconComponent } from './types';
	// Optional: import { Image } from 'lucide-svelte';

	function getDefaultAttributes(baseName = 'Image'): Record<string, any> {
		// Initialize src as null or undefined to trigger lazy loading
		return { name: baseName, src: null, alt: '', assetId: null };
	}

	function getDefaultViewNodeState(): Omit<TweenableNodeState, 'x' | 'y'> {
		return { width: 100, height: 100, scale: 1, rotation: 0 };
	}

	const imageNodePropertySchema: PropertyDefinition[] = [
		// Make src read-only for now as it's managed internally via Object URLs
		{ key: 'src', label: 'Image URL (Read-Only)', type: 'string' }, // Consider making this read-only or hiding it
		{ key: 'alt', label: 'Alt Text', type: 'string' }
	];

	export const nodeTypeDef: Omit<NodeTypeDefinition, 'component'> = {
		ntype: 'image',
		getDefaultAttributes,
		getDefaultViewNodeState,
		displayName: 'Image',
		// icon: Image as IconComponent // Example
		propertySchema: imageNodePropertySchema
	};
</script>

<script lang="ts">
	// INSTANCE SCRIPT
	import type { DataNode, ViewNode } from '$lib/types/types';
	import { currentContextId, availableContextsMap } from '$lib/karta/ContextStore'; // Import availableContextsMap
	import { onMount } from 'svelte';
	import { localAdapter } from '$lib/util/LocalAdapter'; // Import localAdapter

	export let dataNode: DataNode;
	export let viewNode: ViewNode;
	// Check if context exists for this node using the new map
	$: hasContext = $availableContextsMap.has(viewNode.id);

	// Type assertion for attributes
	$: attributes = dataNode.attributes as { src?: string | null; alt?: string; name: string; assetId?: string | null }; // Add assetId type, allow null src
	$: altText = attributes?.alt || `Image for ${attributes?.name ?? dataNode.id}`;
	$: assetId = attributes?.assetId; // Reactive variable for assetId

	// Local state for the image URL and loading status
	let imageUrl: string | null = attributes?.src || null; // Initialize with src if present (might be cached URL from adapter)
	let isLoading = false;

	// Placeholder image (e.g., SVG or data URI) if src is empty
	const placeholderSvg = `
		<svg xmlns="http://www.w3.org/2000/svg" class="w-full h-full text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1">
			<path stroke-linecap="round" stroke-linejoin="round" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
		</svg>
	`;
	const placeholderDataUri = `data:image/svg+xml;utf8,${encodeURIComponent(placeholderSvg)}`;

	async function fetchImageUrl(id: string | null | undefined) {
		if (typeof id !== 'string' || !localAdapter) {
			console.warn(`[ImageNode ${dataNode.id}] Invalid assetId or adapter not ready.`);
			imageUrl = placeholderDataUri;
			return;
		}

		isLoading = true;
		console.log(`[ImageNode ${dataNode.id}] Fetching Object URL for asset ${id}`);
		try {
			const url = await localAdapter.getAssetObjectUrl(id);
			if (url) {
				imageUrl = url;
				// console.log(`[ImageNode ${dataNode.id}] Object URL loaded: ${url.substring(0, 50)}...`);
			} else {
				console.warn(`[ImageNode ${dataNode.id}] Failed to get Object URL for asset ${id}.`);
				imageUrl = placeholderDataUri; // Use placeholder if fetch fails
			}
		} catch (error) {
			console.error(`[ImageNode ${dataNode.id}] Error fetching Object URL:`, error);
			imageUrl = placeholderDataUri; // Use placeholder on error
		} finally {
			isLoading = false;
		}
	}

	// Fetch Object URL on mount if needed
	onMount(() => {
		// Only fetch if imageUrl isn't already set and assetId exists
		if (!imageUrl && assetId) {
			fetchImageUrl(assetId);
		} else if (imageUrl) {
			// console.log(`[ImageNode ${dataNode.id}] Using existing src/cached URL: ${imageUrl.substring(0, 50)}...`);
		}
	});

	// Reactive statement to handle potential changes in assetId after mount
	$: if (assetId && !isLoading && !imageUrl) {
		// Re-trigger fetch if assetId changes and we don't have a URL yet
		console.log(`[ImageNode ${dataNode.id}] AssetId changed or URL missing, re-fetching.`);
		fetchImageUrl(assetId);
	}

	// Determine ring classes based on focal state and context existence
	$: ringClasses = dataNode.id === $currentContextId
		? 'ring-4 ring-offset-2 ring-offset-gray-900 ring-orange-500 rounded' // Focal highlight
		: hasContext
			? 'ring-2 ring-orange-500/50 rounded' // Use border for context outline
			: 'rounded shadow-md'; // Default rounded corners and shadow
</script>

<div
	class={`
		w-full h-full bg-gray-700 flex items-center justify-center overflow-hidden pointer-events-auto
		${ringClasses}
	`}
	title={`Image Node: ${attributes?.name ?? dataNode.id}`}
>
	{#if isLoading}
		<!-- Loading Indicator -->
		<div class="w-full h-full flex items-center justify-center text-gray-400 text-xs animate-pulse">Loading...</div>
	{:else if imageUrl && imageUrl !== placeholderDataUri}
		<!-- Image Display -->
		<img
			src={imageUrl}
			alt={altText}
			class="object-contain w-full h-full pointer-events-none"
			loading="lazy"
			on:error={(e) => {
				// Handle image loading errors (e.g., revoked Object URL)
				console.warn(`[ImageNode ${dataNode.id}] Error loading image src: ${imageUrl?.substring(0, 50)}...`);
				if (e.currentTarget instanceof HTMLImageElement) {
					// Avoid setting placeholder if it's already the placeholder causing the error
					if (e.currentTarget.src !== placeholderDataUri) {
						imageUrl = placeholderDataUri; // Fallback to placeholder on error
					}
				}
			}}
		/>
	{:else}
		<!-- Placeholder Display -->
		<div class="pointer-events-none w-full h-full flex items-center justify-center p-2" title="Image not loaded or found">{@html placeholderSvg}</div>
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