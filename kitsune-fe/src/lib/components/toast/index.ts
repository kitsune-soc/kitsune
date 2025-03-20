import { writable } from 'svelte/store';

import ToastProvider from './ToastProvider.svelte';

type ToastData = {
	severity: 'info' | 'error' | 'success';
	message: string;
};

const toastStore = writable<ToastData[]>([]);

function pushToast(toast: ToastData) {
	toastStore.update((value) => value.concat(toast));
	setTimeout(
		() =>
			toastStore.update((value) => {
				value.shift();
				return value;
			}),
		2_500
	);
}

export { ToastProvider, pushToast, toastStore };
