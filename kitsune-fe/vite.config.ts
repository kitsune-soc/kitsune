import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { SvelteKitPWA, type SvelteKitPWAOptions } from '@vite-pwa/sveltekit';

import houdini from 'houdini/vite';
import { defineConfig } from 'vitest/config';

const pwaOptions: Partial<SvelteKitPWAOptions> = {
	manifest: {
		name: 'Kitsune',
		description: 'Federated social media',
		theme_color: '#ff9e55'
	}
};

export default defineConfig({
	plugins: [houdini(), sveltekit(), SvelteKitPWA(pwaOptions), tailwindcss()],
	server: {
		proxy: {
			'/graphql': 'http://localhost:5000',
			'/oauth/': 'http://localhost:5000',
			'/public': 'http://localhost:5000'
		}
	},
	test: {
		include: ['src/**/*.{test,spec}.{js,ts}']
	}
});
