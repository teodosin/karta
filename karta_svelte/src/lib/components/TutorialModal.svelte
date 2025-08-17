<script lang="ts">
	import { X, ChevronLeft, ChevronRight } from 'lucide-svelte';
	import { fade, fly } from 'svelte/transition';
	import { isTutorialOpen, closeTutorial } from '$lib/stores/TutorialStore';
	import { onMount } from 'svelte';

	let currentSection = 0;
	let sections: Array<{ title: string; content: string }> = [];

	// Simple markdown formatter for bold text
	function formatMarkdown(text: string): string {
		return text.replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>');
	}

	onMount(async () => {
		try {
			const response = await fetch('/tutorial-sections.json');
			sections = await response.json();
		} catch (error) {
			console.error('Failed to load tutorial sections:', error);
			// Fallback sections in case of error
			sections = [
				{
					title: "Welcome to Karta",
					content: "Welcome to Karta! This tutorial failed to load properly, but you can still explore the app by right-clicking to create nodes and experimenting with the interface."
				}
			];
		}
	});

	function nextSection() {
		if (sections.length > 0 && currentSection < sections.length - 1) {
			currentSection++;
		}
	}

	function prevSection() {
		if (sections.length > 0 && currentSection > 0) {
			currentSection--;
		}
	}

	function handleClose() {
		currentSection = 0;
		closeTutorial();
	}

	function handleBackdropClick(event: MouseEvent) {
		if (event.target === event.currentTarget) {
			handleClose();
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			handleClose();
		} else if (event.key === 'ArrowLeft') {
			prevSection();
		} else if (event.key === 'ArrowRight') {
			nextSection();
		}
	}
</script>

<svelte:window on:keydown={handleKeydown} />

{#if $isTutorialOpen}
	<div
		class="fixed inset-0 z-[1000] flex items-center justify-center bg-black bg-opacity-50"
		transition:fade={{ duration: 200 }}
		on:click={handleBackdropClick}
		on:keydown={handleKeydown}
		role="dialog"
		aria-labelledby="tutorial-title"
		aria-modal="true"
		tabindex="-1"
	>
		<div
			class="relative w-full max-w-2xl mx-4 bg-panel-bg border border-orange-400 rounded-lg shadow-2xl"
			transition:fly={{ y: 20, duration: 200 }}
			role="document"
		>
			<!-- Header -->
			<div class="flex items-center justify-between p-6 border-b border-orange-400">
				<div class="flex items-center gap-3">
					<h2 id="tutorial-title" class="text-xl font-semibold text-white">
						{sections.length > 0 ? sections[currentSection].title : 'Tutorial'}
					</h2>
					{#if sections.length > 0}
						<span class="text-sm text-gray-400 bg-gray-700 px-2 py-1 rounded">
							{currentSection + 1} of {sections.length}
						</span>
					{/if}
				</div>
				<button
					on:click={handleClose}
					class="text-gray-400 hover:text-white transition-colors"
					aria-label="Close tutorial"
				>
					<X size={24} />
				</button>
			</div>

			<!-- Content -->
			<div class="p-6">
				{#if sections.length > 0}
					<div class="text-gray-200 whitespace-pre-line leading-relaxed">
						{@html formatMarkdown(sections[currentSection].content)}
					</div>
				{:else}
					<div class="text-gray-400 text-center">Loading tutorial...</div>
				{/if}
			</div>

			<!-- Navigation -->
			{#if sections.length > 0}
				<div class="flex items-center justify-between p-6 border-t border-orange-400">
					<button
						on:click={prevSection}
						disabled={currentSection === 0}
						class="flex items-center gap-2 px-4 py-2 text-sm bg-gray-700 text-white rounded hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
					>
						<ChevronLeft size={16} />
						Previous
					</button>

					<div class="flex gap-2">
						{#each sections as _, index}
							<button
								on:click={() => currentSection = index}
								class="w-3 h-3 rounded-full transition-colors {currentSection === index 
									? 'bg-orange-400' 
									: 'bg-gray-600 hover:bg-gray-500'}"
								aria-label={`Go to section ${index + 1}`}
							></button>
						{/each}
					</div>

					{#if currentSection === sections.length - 1}
						<button
							on:click={handleClose}
							class="px-4 py-2 text-sm bg-orange-500 text-white rounded hover:bg-orange-600 transition-colors"
						>
							Get Started!
						</button>
					{:else}
						<button
							on:click={nextSection}
							class="flex items-center gap-2 px-4 py-2 text-sm bg-orange-500 text-white rounded hover:bg-orange-600 transition-colors"
						>
							Next
							<ChevronRight size={16} />
						</button>
					{/if}
				</div>
			{/if}
		</div>
	</div>
{/if}
