<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLAttributes } from 'svelte/elements';

	let { children, ...props }: HTMLAttributes<HTMLElement> & { children: Snippet } = $props();
</script>

<div class="hero min-h-screen">
	<main {...props} class="hero-content {props.class}">
		{@render children()}
	</main>
</div>
