import forms from '@tailwindcss/forms';
import typography from '@tailwindcss/typography';

import type { Config } from 'tailwindcss';

import { extendTheme } from '../kitsune/theme';

export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],

	theme: {
		extend: {
			...extendTheme
		}
	},

	plugins: [typography, forms]
} as Config;
