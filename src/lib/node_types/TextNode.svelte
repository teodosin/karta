<!--
// --- Karta Runtime Component ---
// This file is planned for inclusion in the MIT-licensed `karta_runtime` package.
// It defines the rendering and behavior for the 'text' node type.
// Play Mode interactions should be handled based on node attributes.
-->
<script context="module" lang="ts">
	// MODULE SCRIPT
	import type { TweenableNodeState, PropertyDefinition } from '$lib/types/types'; // Import PropertyDefinition
	import type { NodeTypeDefinition, IconComponent } from './types';
	// Optional: import { Type } from 'lucide-svelte';

	function getDefaultAttributes(baseName = 'Text'): Record<string, any> {
		return { name: baseName, text: '', fontSize: 16 };
	}

	function getDefaultViewNodeState(): Omit<TweenableNodeState, 'x' | 'y'> {
		return { width: 120, height: 80, scale: 1, rotation: 0 };
	}

	const textNodePropertySchema: PropertyDefinition[] = [
		// { key: 'text', label: 'Content', type: 'textarea' }, // Removed - edit directly on node
		{ key: 'fontSize', label: 'Font Size', type: 'number' } // Assuming number input is desired
	];

	export const nodeTypeDef: Omit<NodeTypeDefinition, 'component'> = {
		ntype: 'text',
		getDefaultAttributes,
		getDefaultViewNodeState,
		displayName: 'Text',
		// icon: Type as IconComponent // Example
		propertySchema: textNodePropertySchema
	};
</script>

<script lang="ts">
	// INSTANCE SCRIPT
	import type { DataNode, ViewNode } from '$lib/types/types';
	import { updateNodeAttributes, currentContextId } from '$lib/karta/KartaStore';
	import { tick, onDestroy } from 'svelte'; // Import onDestroy

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
		if (isEditing) return; // Prevent re-entry if already editing
		editedText = textContent;
		isEditing = true;
		await tick(); // Wait for textarea to render
		textAreaElement?.focus();
		textAreaElement?.select();
		// Add listener for clicks outside
		window.addEventListener('pointerdown', handleClickOutside, { capture: true });
	}

	function handleTextSubmit() {
		if (isEditing && editedText !== textContent) {
			updateNodeAttributes(dataNode.id, { ...attributes, text: editedText });
		}
		isEditing = false;
		removeClickListener(); // Remove listener after submit
	}

	function handleKeyDown(event: KeyboardEvent) {
		// Submit on Ctrl+Enter or Cmd+Enter
		if (event.key === 'Enter' && (event.ctrlKey || event.metaKey)) {
			handleTextSubmit();
		} else if (event.key === 'Escape') {
			isEditing = false; // Cancel editing
			removeClickListener(); // Remove listener on cancel
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

	// --- Click Outside Logic ---
	function handleClickOutside(event: PointerEvent) {
		if (textAreaElement && !textAreaElement.contains(event.target as Node)) {
			handleTextSubmit(); // Submit if click is outside
		}
	}

	function removeClickListener() {
		window.removeEventListener('pointerdown', handleClickOutside, { capture: true });
	}

	// Ensure listener is removed if component is destroyed while editing
	onDestroy(() => {
		if (isEditing) {
			removeClickListener();
		}
	});

	// --- Paste Handler ---
	async function handlePaste(event: ClipboardEvent) {
		if (!isEditing || !textAreaElement || !event.clipboardData) return;

		event.preventDefault(); // Handle paste manually

		const pastedText = event.clipboardData.getData('text/plain');
		if (!pastedText) return;

		const start = textAreaElement.selectionStart;
		const end = textAreaElement.selectionEnd;

		// Construct the new text value
		const newText = editedText.substring(0, start) + pastedText + editedText.substring(end);

		// Update the bound variable - Svelte handles textarea update
		editedText = newText;

		// Wait for Svelte to update the DOM, then set cursor position and adjust height
		await tick();
		if (textAreaElement) {
			const newCursorPos = start + pastedText.length;
			textAreaElement.selectionStart = newCursorPos;
			textAreaElement.selectionEnd = newCursorPos;
			adjustTextareaHeight(); // Adjust height after paste
		}
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
		<!-- Editing State: Textarea -->
		<textarea
			bind:this={textAreaElement}
			bind:value={editedText}
			on:keydown={handleKeyDown}
			on:input={adjustTextareaHeight}
			on:paste={handlePaste}
			class="w-full h-auto bg-yellow-100 outline-none resize-none p-1 leading-tight text-gray-900 block"
			style:font-size="{fontSize}px"
			spellcheck="false"
		></textarea> <!-- Ensure closing tag -->
	{:else}
		<!-- Display State: Div -->
		<div
			class="w-full h-full overflow-y-auto whitespace-pre-wrap break-words p-1 leading-tight"
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
		/* line-height: 1.4; */ /* Removed as leading-tight is used */
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