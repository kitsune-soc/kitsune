<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLDialogAttributes } from 'svelte/elements';

	const {
		children,
		isOpen,
		...rest
	}: { children: Snippet; isOpen: boolean } & HTMLDialogAttributes = $props();

	let dialog: HTMLDialogElement;

	$effect(() => {
		if (isOpen) {
			dialog.showModal();
		} else {
			dialog.close();
		}
	});
</script>

<dialog
	class="prose prose-slate prose-invert rounded bg-dark-1 p-5 backdrop:bg-black/50"
	bind:this={dialog}
	{...rest}
>
	{@render children()}
</dialog>
