<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		class: classNames = '',
		children,
		type = 'primary',
		...rest
	}: {
		class: string;
		type?: string;
		children: Snippet;
	} & any = $props();

	classNames += ` ${type}`;
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
		height: 50px;
		font-size: 16px;

		&:global(.primary) {
			background-color: $shade1dark;

			&:hover {
				background-color: $shade2dark;
			}
		}

		&:global(.secondary) {
			background-color: transparent;
			border: 1px solid;
		}
	}
</style>
