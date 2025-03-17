import { writable } from 'svelte/store';
import { z } from 'zod';

const UI_SETTINGS_KEY = 'ui_settings';
const UI_SETTINGS_SCHEMA = z.object({
	cyberpunkMode: z.boolean()
});
type UiSettingsTy = z.infer<typeof UI_SETTINGS_SCHEMA>;

// attempt to load settings from local storage
const uiSettings = writable<UiSettingsTy>({ cyberpunkMode: false }, (set) => {
	const maybeSettings = localStorage.getItem(UI_SETTINGS_KEY);
	if (!maybeSettings) {
		return;
	}

	const parseResult = UI_SETTINGS_SCHEMA.safeParse(JSON.parse(maybeSettings));
	if (parseResult.success) {
		set(parseResult.data);
	} else {
		console.log('failed to load settings from local storage');
	}
});

// store in local storage on change
uiSettings.subscribe((newSettings) => {
	localStorage.setItem(UI_SETTINGS_KEY, JSON.stringify(newSettings));
});

export { uiSettings };
