import { paraglide } from '@inlang/paraglide-sveltekit/vite';
import { sveltekit } from '@sveltejs/kit/vite';

import houdini from 'houdini/vite';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [
		houdini(),
		sveltekit(),
		paraglide({
			project: './project.inlang',
			outdir: './src/lib/paraglide'
		})
	],
	server: {
		proxy: {
			'/graphql': 'http://localhost:5000'
		}
	},
	test: {
		include: ['src/**/*.{test,spec}.{js,ts}']
	}
});
