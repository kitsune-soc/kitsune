<script lang="ts">
	import Icon from '@iconify/svelte';

	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	let {
		children,
		buttonType = 'primary',
		loading = false,
		...props
	}: {
		/**
		 * The type of button to render.
		 *
		 * @default 'primary'
		 */
		buttonType?: 'primary' | 'secondary' | 'danger';
		loading?: boolean;
		children: Snippet;
	} & HTMLButtonAttributes = $props();

	let disabled = $derived(props.disabled || loading);
</script>

<button
	{...props}
	class="min-h-1 rounded p-2 transition duration-500 {buttonType} {props.class}"
	class:cursor-not-allowed={disabled}
	{disabled}
>
	{#if loading}
		<Icon class="m-auto h-auto w-8" icon="line-md:loading-loop" />
	{:else}
		{@render children()}
	{/if}
</button>

<style>
	.primary {
		@apply bg-shade1-dark hover:enabled:bg-shade2-dark disabled:text-dark-1 [&:not(:hover)]:text-dark-1;
	}

	.secondary {
		@apply border-2 border-solid bg-transparent;
	}

	.danger {
		@apply bg-red-700 text-white;
	}
</style>
