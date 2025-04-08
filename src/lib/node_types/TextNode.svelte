<script context="module" lang="ts">
	// MODULE SCRIPT
	import type { TweenableNodeState } from '$lib/types/types';
	import type { NodeTypeDefinition, IconComponent } from './types';
	// Optional: import { Type } from 'lucide-svelte';

	function getDefaultAttributes(baseName = 'Text'): Record<string, any> {
		return { name: baseName, text: '', fontSize: 16 };
	}

	function getDefaultViewNodeState(): Omit<TweenableNodeState, 'x' | 'y'> {
		return { width: 120, height: 80, scale: 1, rotation: 0 };
	}

	export const nodeTypeDef: Omit<NodeTypeDefinition, 'component'> = {
		ntype: 'text',
		getDefaultAttributes,
		getDefaultViewNodeState,
		displayName: 'Text'
		// icon: Type as IconComponent // Example
	};
</script>

<script lang="ts">
	// INSTANCE SCRIPT
	import type { DataNode, ViewNode } from '$lib/types/types';
	import { updateNodeAttributes, currentContextId } from '$lib/karta/KartaStore';
	import { tick } from 'svelte';

	export let dataNode: DataNode;
	export let viewNode: ViewNode;

	// Type assertion for attributes - assumes NodeWrapper ensures correct ntype/attributes
	$: attributes = dataNode.attributes as { text?: string; fontSize?: number; name: string };
	$: textContent = attributes?.text ?? '';
	$: fontSize = attributes?.fontSize ?? 16; // Default font size if not specified

	let isEditing = false;
	let editedText = '';
	let textAreaElement: HTMLTextAreaElement | null = null;

	async function startEditing() {
		editedText = textContent;
		isEditing = true;
		await tick(); // Wait for textarea to render
		textAreaElement?.focus();
		textAreaElement?.select();
	}

	function handleTextSubmit() {
		if (isEditing && editedText !== textContent) {
			updateNodeAttributes(dataNode.id, { ...attributes, text: editedText });
		}
		isEditing = false;
	}

	function handleKeyDown(event: KeyboardEvent) {
		// Submit on Ctrl+Enter or Cmd+Enter
		if (event.key === 'Enter' && (event.ctrlKey || event.metaKey)) {
			handleTextSubmit();
		} else if (event.key === 'Escape') {
			isEditing = false; // Cancel editing
		}
	}

	// Adjust textarea height dynamically (simple approach)
	function adjustTextareaHeight() {
		if (textAreaElement) {
			textAreaElement.style.height = 'auto'; // Reset height
			textAreaElement.style.height = `${textAreaElement.scrollHeight}px`;
		}
	}

	$: if (isEditing && textAreaElement) {
		adjustTextareaHeight(); // Adjust height when editing starts or text changes
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class={`
		w-full h-full bg-yellow-100 text-gray-900 p-2 overflow-hidden
		flex items-center justify-center pointer-events-auto
		${dataNode.id === $currentContextId ? 'ring-4 ring-offset-2 ring-offset-gray-900 ring-orange-500 rounded' : 'rounded shadow-md'}
	`}
	title={`Text Node: ${attributes?.name ?? dataNode.id}`}
	on:dblclick={startEditing}
>
	{#if isEditing}
		<!-- svelte-ignore element_invalid_self_closing_tag -->
		<textarea
			bind:this={textAreaElement}
			bind:value={editedText}
			on:blur={handleTextSubmit}
			on:keydown={handleKeyDown}
			on:input={adjustTextareaHeight}
			class="w-full h-auto bg-white border border-blue-400 rounded outline-none resize-none p-1 leading-tight"
			style:font-size="{fontSize}px"
			spellcheck="false"
		/>
	{:else}
		<!-- Display Text - Use whitespace-pre-wrap to respect newlines -->
		<div
			class="w-full h-full overflow-y-auto whitespace-pre-wrap break-words"
			style:font-size="{fontSize}px"
		>
			{textContent || ''} {#if !textContent}&nbsp;{/if} <!-- Ensure div has height even if empty -->
		</div>
	{/if}
</div>

<style>
	div, textarea {
		box-sizing: border-box;
		font-family: sans-serif; /* Or choose a specific font */
		line-height: 1.4; /* Adjust line height for readability */
	}
	/* Add scrollbar styling if desired */
	div::-webkit-scrollbar {
		width: 6px;
	}
	div::-webkit-scrollbar-thumb {
		background-color: rgba(0,0,0,0.2);
		border-radius: 3px;
	}
	textarea::-webkit-scrollbar {
		width: 6px;
	}
	textarea::-webkit-scrollbar-thumb {
		background-color: rgba(0,0,0,0.3);
		border-radius: 3px;
	}
</style>