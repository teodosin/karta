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
		return {
			name: baseName, // Stays unprefixed
			type_src: null, // Core content attribute (will hold Object URL)
			type_alt: '', // Core content attribute
			type_assetId: null, // ID to fetch the actual asset blob
			view_isNameVisible: true // Generic view default
		};
	}

	function getDefaultViewNodeState(): Omit<TweenableNodeState, 'x' | 'y'> {
		return { width: 100, height: 100, scale: 1, rotation: 0 };
	}

	const imageNodePropertySchema: PropertyDefinition[] = [
		// Make type_src read-only as it's managed internally via Object URLs
		{ key: 'type_src', label: 'Image URL (Read-Only)', type: 'string' },
		{ key: 'type_alt', label: 'Alt Text', type: 'string' }
	];

	export const nodeTypeDef: Omit<NodeTypeDefinition, 'component'> = {
		ntype: 'core/image',
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
	$: attributes = dataNode.attributes as {
		name: string;
		type_src?: string | null;
		type_alt?: string;
		type_assetId?: string | null;
		view_isNameVisible?: boolean;
	};
	$: altText = attributes?.type_alt || `Image for ${attributes?.name ?? dataNode.id}`;
	$: assetId = attributes?.type_assetId; // Reactive variable for assetId

	// Local state for the image URL and loading status
	let imageUrl: string | null = attributes?.type_src || null; // Initialize with type_src if present
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
		try {
			const url = await localAdapter.getAssetObjectUrl(id);
			if (url) {
				imageUrl = url;
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

	// This reactive block ensures the imageUrl is correctly determined whenever
	// the component's props change. It handles both filesystem-based nodes (with a path)
	// and asset-based nodes (pasted/dropped).
	$: {
		if (dataNode.path) {
			// Filesystem node: construct the URL directly to the asset endpoint.
			// The path from the dataNode might include the "vault/" prefix, which needs to be stripped
			// as the backend will join it with its own vault root path.
			let relativePath = dataNode.path;
			if (relativePath.startsWith('vault/')) {
				relativePath = relativePath.substring('vault/'.length);
			}
			const newUrl = `/api/asset/${encodeURI(relativePath)}`;
			if (imageUrl !== newUrl) {
				imageUrl = newUrl;
			}
		} else if (assetId && !isLoading && !imageUrl) {
			// Pasted/dropped asset: use the existing fetch logic.
			fetchImageUrl(assetId);
		}
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
		w-full h-full bg-transparent flex items-center justify-center overflow-hidden pointer-events-auto
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