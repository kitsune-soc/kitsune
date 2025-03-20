<script lang="ts">
	import type { Snippet } from 'svelte';
	import { fade } from 'svelte/transition';

	import { toastStore } from '.';

	let { children }: { children: Snippet } = $props();
</script>

<div class="toast toast-center z-50">
	{#each $toastStore as toast, idx ([idx, toast])}
		<div
			transition:fade
			class="alert"
			class:alert-error={toast.severity === 'error'}
			class:alert-info={toast.severity === 'info'}
			class:alert-success={toast.severity === 'success'}
		>
			{toast.message}
		</div>
	{/each}
</div>

{@render children()}
