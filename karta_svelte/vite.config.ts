import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import path from 'node:path';

import type { UserConfig } from 'vite';

const config: UserConfig = {
	plugins: [tailwindcss(), sveltekit()],
	resolve: {
		alias: {
			$lib: path.resolve('./src/lib'),
			$viewport: path.resolve('./src/lib/karta/ViewportStore.ts')
		},
		dedupe: ['svelte']
	}
};

export default defineConfig(config);
