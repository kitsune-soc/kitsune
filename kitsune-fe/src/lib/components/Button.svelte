<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	let {
		class: classNames = '',
		children,
		buttonType = 'primary',
		...rest
	}: {
		class?: string;
		/**
		 * The type of button to render.
		 *
		 * @default 'primary'
		 */
		buttonType?: 'primary' | 'secondary';
		children: Snippet;
	} & HTMLButtonAttributes = $props();

	if (buttonType === 'primary') {
		classNames += ` primary`; // ToDo: Port styles to tailwind
	} else if (buttonType === 'secondary') {
		classNames += ` border-solid border-2 bg-transparent`;
	}
</script>

<button
	class={classNames + ` min-h-1 cursor-pointer rounded p-2 transition duration-500`}
	{...rest}
>
	{@render children()}
</button>

<style lang="scss">
	@use '../../styles/colours' as *;

	button:global(.primary) {
		background-color: $shade1dark;

		&:not(:hover) {
			color: $dark1;
		}

		&:hover:not([disabled]) {
			background-color: $shade2dark;
		}
	}
</style>
