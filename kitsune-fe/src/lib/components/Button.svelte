<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	let {
		class: classNames = '',
		children,
		buttonType = 'primary',
		...rest
	}: {
		class: string;
		/**
		 * The type of button to render.
		 *
		 * @default 'primary'
		 */
		buttonType?: string;
		children: Snippet;
	} & HTMLButtonAttributes = $props();

	classNames += ` ${buttonType}`;
</script>

<button class={classNames} {...rest}>
	{@render children()}
</button>

<style lang="scss">
	@use '../../styles/colours' as *;

	button {
		transition: 0.5s;
		cursor: pointer;

		border: 0;
		border-radius: 10px;

		padding: 10px;
		width: 100px;
		min-height: 50px;
		font-size: 16px;

		&:global(.primary) {
			background-color: $shade1dark;

			&:not(:hover) {
				color: $dark1;
			}

			&:hover {
				background-color: $shade2dark;
			}
		}

		&:global(.secondary) {
			border: 1px solid;
			background-color: transparent;
		}
	}
</style>
