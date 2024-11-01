import forms from '@tailwindcss/forms';
import typography from '@tailwindcss/typography';

import type { Config } from 'tailwindcss';

export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],

	theme: {
		extend: {
			colors: {
				dark: {
					'1': '#1c1626',
					'2': '#2b233a',
					'3': '#042f40'
				}
				// ToDo: Port rest of colour palette over
			}
		}
	},

	plugins: [typography, forms]
} as Config;
