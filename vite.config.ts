import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

import type { UserConfig } from 'vite';

const config: UserConfig = {
	plugins: [tailwindcss(), sveltekit()]
};

export default defineConfig(config);
