<script lang="ts">
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
		buttonType?: 'primary' | 'neutral' | 'error';
		loading?: boolean;
		children: Snippet;
	} & HTMLButtonAttributes = $props();

	let disabled = $derived(props.disabled || loading);
</script>

<!-- Doing this the ugly ahh way because otherwise the TailwindCSS compiler won't include the utility classes in the bundle -->
<button
	{...props}
	class="btn {props.class}"
	class:btn-primary={buttonType === 'primary'}
	class:btn-neutral={buttonType === 'neutral'}
	class:btn-error={buttonType === 'error'}
	{disabled}
>
	{#if loading}
		<span class="loading loading-spinner"></span>
	{:else}
		{@render children()}
	{/if}
</button>
