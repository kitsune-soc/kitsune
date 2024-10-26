import forms from '@tailwindcss/forms';
import typography from '@tailwindcss/typography';

import type { Config } from 'tailwindcss';

export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],

	theme: {
		extend: {
			colors: {
				dark: {
					'1': '#2b233a'
				}
			}
		}
	},

	plugins: [typography, forms]
} as Config;
