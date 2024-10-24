import { sveltekit } from '@sveltejs/kit/vite';

import houdini from 'houdini/vite';
import Icons from 'unplugin-icons/vite';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [
		houdini(),
		sveltekit(),
		Icons({
			autoInstall: true,
			compiler: 'svelte'
		})
	],
	test: {
		include: ['src/**/*.{test,spec}.{js,ts}']
	}
});
