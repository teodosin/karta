<script lang="ts">
	import { base } from '$app/paths';
	import { Download, ExternalLink, Network, FileText, Search, Layers } from 'lucide-svelte';
</script>

<svelte:head>
	<title>Karta - The Canvas File Browser</title>
	<meta name="description" content="Karta is the canvas file browser. Organize visually and by association. No importing necessary." />
</svelte:head>

<!-- Hero Section -->
<section class="min-h-screen text-white flex items-center" style="background: linear-gradient(to bottom right, var(--color-viewport-bg), var(--color-panel-bg), var(--color-panel-hl));">
	<div class="container mx-auto px-6">
		<div class="max-w-4xl mx-auto text-center">
			<h1 class="text-5xl md:text-7xl font-bold mb-6" style="background: linear-gradient(to right, var(--color-focal-hl), var(--color-contrast-color)); -webkit-background-clip: text; background-clip: text; color: transparent;">
				Karta
			</h1>
			<p class="text-xl md:text-2xl mb-4 leading-relaxed font-medium" style="color: var(--color-text-color);">
				The Canvas File Browser
			</p>
			<p class="text-lg mb-8 text-gray-300 leading-relaxed max-w-2xl mx-auto">
				Organize visually and by association. No importing necessary.
			</p>
			<div class="border rounded-lg p-4 mb-8 max-w-2xl mx-auto" style="background-color: rgba(245, 158, 11, 0.1); border-color: rgba(245, 158, 11, 0.4);">
				<p class="text-yellow-200 text-sm">
					⚠️ <strong>Experimental Release:</strong> This is the first alpha version and is not yet usable as a file browser replacement. 
					<span class="text-yellow-100">But it's a glimpse into the future of file organization—try it and see what's possible!</span>
				</p>
			</div>
			<div class="flex flex-col sm:flex-row gap-4 justify-center items-center mb-12">
				<a 
					href="https://github.com/teodosin/karta_ecosystem/releases" 
					class="inline-flex items-center gap-2 text-white font-semibold py-3 px-6 rounded-lg transition-colors duration-200 hover:opacity-80"
					style="background-color: var(--color-focal-hl);"
				>
					<Download size={20} />
					Download Alpha 0.1.0
				</a>
				<a 
					href="https://github.com/teodosin/karta_ecosystem" 
					class="inline-flex items-center gap-2 border-2 text-gray-300 hover:text-white font-semibold py-3 px-6 rounded-lg transition-colors duration-200 hover:border-gray-400"
					style="border-color: var(--color-connection-color);"
				>
					<ExternalLink size={20} />
					View on GitHub
				</a>
			</div>
			<p class="text-sm text-gray-400">
				Available for macOS, Windows, and Linux
			</p>
		</div>
	</div>
</section>

<!-- Features Section -->
<section class="py-20" style="background-color: var(--color-viewport-bg);">
	<div class="container mx-auto px-6">
		<div class="max-w-6xl mx-auto">
			<h2 class="text-4xl font-bold text-center mb-4" style="color: var(--color-text-color);">
				Organize. Visualize. Create.
			</h2>
			<p class="text-xl text-center mb-16 text-gray-400 max-w-3xl mx-auto">
				Transform how you manage files with a canvas-based approach that reveals connections and relationships.
			</p>
			
			<div class="grid md:grid-cols-3 gap-8 mb-16">
				<div class="text-center p-6 rounded-lg" style="background-color: var(--color-panel-bg);">
					<div class="w-16 h-16 rounded-full flex items-center justify-center mx-auto mb-4" style="background-color: rgba(244, 144, 45, 0.1);">
						<Network style="color: var(--color-focal-hl);" size={32} />
					</div>
					<h3 class="text-xl font-semibold mb-3" style="color: var(--color-text-color);">Node-Based Organization</h3>
					<p class="text-gray-400">
						Organize files, notes, and ideas as nodes in a visual graph. Connect related elements to see the bigger picture.
					</p>
				</div>
				
				<div class="text-center p-6 rounded-lg" style="background-color: var(--color-panel-bg);">
					<div class="w-16 h-16 rounded-full flex items-center justify-center mx-auto mb-4" style="background-color: rgba(96, 165, 250, 0.1);">
						<FileText style="color: var(--color-contrast-color);" size={32} />
					</div>
					<h3 class="text-xl font-semibold mb-3" style="color: var(--color-text-color);">File System Integration</h3>
					<p class="text-gray-400">
						Work with your existing files. Organize visually and by association. No importing necessary.
					</p>
				</div>
				
				<div class="text-center p-6 rounded-lg" style="background-color: var(--color-panel-bg);">
					<div class="w-16 h-16 rounded-full flex items-center justify-center mx-auto mb-4" style="background-color: rgba(116, 31, 47, 0.3);">
						<Layers style="color: var(--color-panel-hl);" size={32} />
					</div>
					<h3 class="text-xl font-semibold mb-3" style="color: var(--color-text-color);">Contextual Views</h3>
					<p class="text-gray-400">
						Switch between different contexts to focus on specific aspects of your project while maintaining the whole.
					</p>
				</div>
			</div>

			<!-- Screenshots placeholder -->
			<div class="rounded-lg shadow-lg p-8 mb-16 border" style="background-color: var(--color-panel-bg); border-color: var(--color-panel-hl);">
				<h3 class="text-2xl font-semibold mb-6 text-center" style="color: var(--color-text-color);">
					See Karta in Action
				</h3>
				<div class="grid md:grid-cols-2 gap-8">
					<div class="rounded-lg aspect-video flex items-center justify-center border" style="background-color: var(--color-viewport-bg); border-color: var(--color-connection-color);">
						<p style="color: var(--color-text-color);">Project Graph View</p>
					</div>
					<div class="rounded-lg aspect-video flex items-center justify-center border" style="background-color: var(--color-viewport-bg); border-color: var(--color-connection-color);">
						<p style="color: var(--color-text-color);">Contextual Node Organization</p>
					</div>
				</div>
				<p class="text-center text-gray-400 mt-4">
					<em>Screenshots coming soon - Karta is currently in early alpha</em>
				</p>
			</div>
		</div>
	</div>
</section>

<!-- Roadmap Section -->
<section class="py-20 text-white" style="background-color: var(--color-panel-bg);">
	<div class="container mx-auto px-6">
		<div class="max-w-6xl mx-auto">
			<h2 class="text-4xl font-bold text-center mb-4">
				What's Coming Next
			</h2>
			<p class="text-xl text-center mb-12 text-gray-300">
				Karta is actively being developed. Here's what's on the horizon:
			</p>
			
			<div class="grid md:grid-cols-3 gap-6">
				<div class="rounded-lg p-6 border" style="background-color: var(--color-panel-hl); border-color: var(--color-connection-color); border-opacity: 0.5;">
					<div class="w-8 h-8 rounded-full flex items-center justify-center mb-4" style="background-color: var(--color-connection-color);">
						<span class="text-sm font-bold text-white">1</span>
					</div>
					<h3 class="text-xl font-semibold mb-3">File Browser Features</h3>
					<p class="text-gray-300">
						Advanced file operations like opening files in external applications, file previews, search, and bulk operations.
					</p>
				</div>
				
				<div class="rounded-lg p-6 border" style="background-color: var(--color-panel-hl); border-color: var(--color-contrast-color); border-opacity: 0.3;">
					<div class="w-8 h-8 rounded-full flex items-center justify-center mb-4" style="background-color: var(--color-contrast-color);">
						<span class="text-sm font-bold text-white">2</span>
					</div>
					<h3 class="text-xl font-semibold mb-3">History Tracking</h3>
					<p class="text-gray-300">
						Freely undo and redo changes so you can experiment without fear.
					</p>
				</div>
				
				<div class="rounded-lg p-6 border" style="background-color: var(--color-panel-hl); border-color: var(--color-focal-hl); border-opacity: 0.3;">
					<div class="w-8 h-8 rounded-full flex items-center justify-center mb-4" style="background-color: var(--color-focal-hl);">
						<span class="text-sm font-bold text-white">3</span>
					</div>
					<h3 class="text-xl font-semibold mb-3">Performance Improvements</h3>
					<p class="text-gray-300">
						Optimizing graph rendering and interaction for larger file systems with hundreds of files.
					</p>
				</div>
			</div>
			
			<div class="text-center mt-12">
				<p class="text-gray-300 mb-4">
					Want to stay updated on development progress?
				</p>
				<a 
					href="https://github.com/teodosin/karta_ecosystem" 
					class="inline-flex items-center gap-2 font-semibold hover:opacity-80 transition-opacity"
					style="color: var(--color-focal-hl);"
				>
					<ExternalLink size={20} />
					Follow on GitHub
				</a>
			</div>
		</div>
	</div>
</section>

<!-- Footer -->
<footer class="py-12 text-gray-400" style="background-color: var(--color-viewport-bg);">
	<div class="container mx-auto px-6">
		<div class="max-w-4xl mx-auto text-center">
			<h3 class="text-2xl font-bold mb-4" style="color: var(--color-text-color);">Karta</h3>
			<p class="mb-6">
				The canvas file browser for visual organization.
			</p>
			<div class="flex justify-center gap-6 mb-6">
				<a href="https://github.com/teodosin/karta_ecosystem" class="transition-colors" style="color: var(--color-text-color); hover: var(--color-focal-hl);">
					GitHub
				</a>
				<a href="mailto:karta@teodosin.com" class="transition-colors" style="color: var(--color-text-color);">
					Contact
				</a>
			</div>
			<p class="text-sm">
				© 2025 Karta. Licensing in progress.
			</p>
		</div>
	</div>
</footer>
