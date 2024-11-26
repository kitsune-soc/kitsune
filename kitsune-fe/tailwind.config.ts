import forms from '@tailwindcss/forms';
import typography from '@tailwindcss/typography';
import { extendTheme } from '../kitsune/theme';

import type { Config } from 'tailwindcss';

export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],

	theme: {
		extend: {
			...extendTheme
		}
	},

	plugins: [typography, forms]
} as Config;
