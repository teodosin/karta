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
		return {
			name: baseName,
			text: '',
			karta_fontSize: 16, // Default font size
			karta_fillColor: '#FEF9C3', // Default tan post-it color
			karta_textColor: '#000000', // Default black text
			karta_font: 'Nunito'       // Default font (matches global default)
		};
	}

	function getDefaultViewNodeState(): Omit<TweenableNodeState, 'x' | 'y'> {
		return { width: 120, height: 80, scale: 1, rotation: 0 };
	}

	const textNodePropertySchema: PropertyDefinition[] = [
		// { key: 'text', label: 'Content', type: 'textarea' }, // Removed - edit directly on node
		// { key: 'karta_fontSize', label: 'Font Size', type: 'number' } // Removed - Handled in View Styles
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
	import type { DataNode, ViewNode, AvailableFont } from '$lib/types/types'; // Import AvailableFont
	import { updateNodeAttributes } from '$lib/karta/NodeStore'; // Corrected import
	import { currentContextId } from '$lib/karta/ContextStore'; // Corrected import
	import { tick, onDestroy } from 'svelte'; // Import onDestroy
	import { AVAILABLE_FONTS } from '$lib/types/types'; // Import AVAILABLE_FONTS

	export let dataNode: DataNode;
	export let viewNode: ViewNode;

	// Type assertion for dataNode attributes - assumes NodeWrapper ensures correct ntype/attributes
	$: dataNodeAttributes = dataNode.attributes as { text?: string; karta_fontSize?: number; name: string; karta_fillColor?: string; karta_textColor?: string; karta_font?: AvailableFont };
	$: textContent = dataNodeAttributes?.text ?? '';

	// Type assertion for viewNode attributes
	$: viewNodeAttributes = viewNode.attributes as { karta_fontSize?: number; karta_fillColor?: string; karta_textColor?: string; karta_font?: AvailableFont } | undefined;

	// --- Define Fallbacks ---
	const FALLBACK_FILL_COLOR = '#FEF9C3'; // Default tan post-it color (Tailwind yellow-100)
	const FALLBACK_TEXT_COLOR = '#000000'; // Default black text
	const FALLBACK_FONT: AvailableFont = 'Nunito'; // Use global default font as fallback
	const FALLBACK_FONT_SIZE = 16; // Default font size

	// --- Reactive Effective Styles ---
	$: effectiveFillColor = viewNodeAttributes?.karta_fillColor ?? dataNodeAttributes?.karta_fillColor ?? FALLBACK_FILL_COLOR;
	$: effectiveTextColor = viewNodeAttributes?.karta_textColor ?? dataNodeAttributes?.karta_textColor ?? FALLBACK_TEXT_COLOR;
	$: effectiveFont = viewNodeAttributes?.karta_font ?? dataNodeAttributes?.karta_font ?? FALLBACK_FONT;
	$: effectiveFontSize = viewNodeAttributes?.karta_fontSize ?? dataNodeAttributes?.karta_fontSize ?? FALLBACK_FONT_SIZE;


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
			// Only update the 'text' attribute on the DataNode
			updateNodeAttributes(dataNode.id, { text: editedText });
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

	// Synchronize editedText with textContent when not editing or when textContent changes
	$: if (!isEditing) {
		editedText = textContent;
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
		}
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class={`
		w-full h-full p-2 overflow-hidden
		flex items-center justify-center pointer-events-auto
		${dataNode.id === $currentContextId ? 'ring-4 ring-offset-2 ring-offset-gray-900 ring-orange-500 rounded' : 'rounded shadow-md'}
	`}
	style:background-color={effectiveFillColor}
	style:color={effectiveTextColor}
	title={`Text Node: ${dataNodeAttributes?.name ?? dataNode.id}`}
	on:dblclick={startEditing}
>
	{#if isEditing}
		<!-- Editing State: Textarea -->
		<textarea
			bind:this={textAreaElement}
			bind:value={editedText}
			on:keydown={handleKeyDown}
			on:paste={handlePaste}
			class="w-full h-full outline-none resize-none leading-tight block"
			style:font-size="{effectiveFontSize}px"
			style:background-color={effectiveFillColor}
			style:color={effectiveTextColor}
			style:font-family={effectiveFont}
			spellcheck="false"
		></textarea> <!-- Ensure closing tag -->
	{:else}
		<!-- Display State: Div -->
		<div
			class="w-full h-full overflow-y-auto whitespace-pre-wrap break-words leading-tight"
			style:font-size="{effectiveFontSize}px"
			style:font-family={effectiveFont}
		>
			{textContent || ''} {#if !textContent}&nbsp;{/if} <!-- Ensure div has height even if empty -->
		</div>
	{/if}
</div>

<style>
	div, textarea {
		box-sizing: border-box;
		/* font-family is now set via inline style */
		/* line-height: 1.4; */ /* Removed as leading-tight is used */
		transition: background-color 0.2s ease, color 0.2s ease; /* Add transitions */
	}
	/* Add scrollbar styling if desired */
	div::-webkit-scrollbar {
		width: 4px;
	}
	div::-webkit-scrollbar-thumb {
		background-color: rgba(0,0,0,0.2);
		border-radius: 3px;
	}
	textarea::-webkit-scrollbar {
		width: 3px;
	}
	textarea::-webkit-scrollbar-thumb {
		background-color: rgba(0,0,0,0.3);
		border-radius: 3px;
	}
</style>