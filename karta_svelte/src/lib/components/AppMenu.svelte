<script lang="ts">
	import { Menu, X } from "lucide-svelte";
	import { fly } from "svelte/transition";
	import { localAdapter } from "$lib/util/LocalAdapter"; // Import localAdapter
	import ThemeEditor from "./ThemeEditor.svelte";
	import { settings } from "$lib/karta/SettingsStore";

	let isOpen = false;
	let fileInput: HTMLInputElement | null = null; // Reference for the hidden file input

	// Function to trigger file input click
	const handleImportClick = () => {
		fileInput?.click(); // Trigger click on the hidden input
	};

	// Function to handle file selection and processing
	const handleFileSelected = async (event: Event) => {
		const target = event.target as HTMLInputElement;
		const file = target.files?.[0];

		if (!file) {
			isOpen = false;
			return;
		}

		isOpen = false; // Close menu immediately

		if (!localAdapter) {
			console.error("LocalAdapter not initialized.");
			alert("Error: Database connection not available.");
			return;
		}

		const reader = new FileReader();

		reader.onload = async (e) => {
			const text = e.target?.result;
			if (typeof text !== "string") {
				alert("Error reading file content.");
				return;
			}

			try {
				const importData = JSON.parse(text);
				// Basic validation (can be expanded)
				if (
					!importData ||
					importData.version !== 1 ||
					!Array.isArray(importData.nodes)
				) {
					throw new Error("Invalid file format or version.");
				}

				// Confirmation before overwriting
				if (
					!confirm(
						"Importing will replace ALL existing data. Are you sure?",
					)
				) {
					// Reset file input value so the same file can be selected again
					if (target) target.value = "";
					return;
				}

				// Add null check for localAdapter inside the callback
				if (!localAdapter) {
					console.error("LocalAdapter became null during file read.");
					alert("Error: Database connection lost during import.");
					if (target) target.value = ""; // Reset input
					return;
				}

				await localAdapter.importData(importData);
				alert(
					"Import successful! Please reload the page to see the changes.",
				);
				// Consider a more robust state refresh mechanism than reload later
			} catch (error: any) {
				console.error("Error processing or importing file:", error);
				alert(
					`Error importing file: ${error.message || "Unknown error"}`,
				);
			} finally {
				// Reset file input value so the same file can be selected again
				if (target) target.value = "";
			}
		};

		reader.onerror = (e) => {
			console.error("Error reading file:", e);
			alert("Error reading file.");
			// Reset file input value
			if (target) target.value = "";
		};

		reader.readAsText(file);
	};

	const handleExport = async () => {
		if (!localAdapter) {
			console.error("LocalAdapter not initialized.");
			alert("Error: Database connection not available.");
			isOpen = false;
			return;
		}
		try {
			const exportData = await localAdapter.getExportData();

			// Create JSON string
			const jsonString = JSON.stringify(exportData, null, 2); // Pretty print JSON

			// Create Blob
			const blob = new Blob([jsonString], { type: "application/json" });

			// Create Object URL
			const url = URL.createObjectURL(blob);

			// Create temporary link
			const a = document.createElement("a");
			a.href = url;
			// Create filename with timestamp
			const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
			a.download = `karta-export-${timestamp}.json`;

			// Trigger download
			document.body.appendChild(a); // Append link to body
			a.click(); // Programmatically click the link

			// Clean up
			document.body.removeChild(a); // Remove link from body
			// Revoke URL after a short delay to ensure download starts
			setTimeout(() => URL.revokeObjectURL(url), 100);

			// Optional: Provide user feedback (e.g., using a toast notification library later)
			// alert('Export complete!'); // Simple feedback for now
		} catch (error) {
			console.error("Error exporting data:", error);
			alert("Error exporting data. Check console for details.");
		} finally {
			isOpen = false; // Close menu after action
		}
	};

	function toggleMenu() {
		isOpen = !isOpen;
	}
</script>

<div class="fixed top-2 left-2 z-[60]">
<!-- Standard HTML button with Tailwind classes for styling -->
<div class="flex items-center gap-2 m-2">
	<button
		on:click={toggleMenu}
		type="button"
		class="inline-flex items-center justify-center rounded-md p-1 text-foreground focus:outline-none focus:ring-2 focus:ring-inset"
		style="--panel-hl: {$settings.colorTheme[
			'panel-hl'
		]}; --focal-hl: {$settings.colorTheme['focal-hl']};"
		aria-label={isOpen ? "Close Menu" : "Open Menu"}
	>
		{#if isOpen}
			<X class="h-5 w-5" />
		{:else}
			<Menu class="h-5 w-5" />
		{/if}
	</button>
	<span class="text-sm text-gray-300 font-mono">Karta Alpha 0.1.0</span>
</div>

	{#if isOpen}
		<div
			class="absolute left-0 z-[500] top-full m-2 w-80 rounded-md border border-orange-400 bg-panel-bg p-2 text-white shadow-lg"
			transition:fly={{ y: -5, duration: 150 }}
		>
			<!-- Hidden file input -->
			<input
				type="file"
				bind:this={fileInput}
				class="hidden"
				accept=".json"
				on:change={handleFileSelected}
			/>
			<!-- Standard HTML buttons for menu items -->
			<!-- Changed on:click to trigger input click -->
			<div class="my-2 border-t border-orange-400" />
			<ThemeEditor />
			<div
				class="mt-2 border-t border-fuchsia-800 pt-2 text-sm text-fuchsia-100"
			>
				Have any feedback or suggestions for future versions? Get in
				touch at
				<a
					href="mailto:karta@teodosin.com"
					class="underline hover:text-white"
				>
					karta@teodosin.com
				</a>
			</div>
		</div>
	{/if}
</div>
