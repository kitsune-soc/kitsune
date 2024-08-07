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

<dialog bind:this={dialog} {...rest}>
	{@render children()}
</dialog>

<style lang="scss">
	@use '../../styles/colours' as *;

	dialog {
		background-color: $dark1;

		border-width: 0px;
		border-radius: 5px;
	}

	::backdrop {
		background-color: black;
		opacity: 50%;
	}
</style>
