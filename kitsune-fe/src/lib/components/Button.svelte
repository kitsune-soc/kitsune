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
		buttonType?: 'primary' | 'secondary' | 'danger';
		children: Snippet;
	} & HTMLButtonAttributes = $props();

	if (buttonType === 'primary') {
		classNames += ` [&:not(:hover)]:text-dark-1 bg-shade1-dark hover:enabled:bg-shade2-dark`;
	} else if (buttonType === 'secondary') {
		classNames += ` border-solid border-2 bg-transparent`;
	} else if (buttonType === 'danger') {
		classNames += ` bg-red-700 text-white`;
	}
</script>

<button
	class={classNames + ` min-h-1 cursor-pointer rounded p-2 transition duration-500`}
	{...rest}
>
	{@render children()}
</button>
