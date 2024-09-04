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
		border-width: 0px;
		border-radius: 5px;
		background-color: $dark1;
	}

	::backdrop {
		opacity: 50%;
		background-color: black;
	}
</style>
