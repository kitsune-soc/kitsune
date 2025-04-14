<script lang="ts">
	import { createDialog, melt } from '@melt-ui/svelte';

	import type { Snippet } from 'svelte';
	import { fade } from 'svelte/transition';

	let { children }: { children: Snippet } = $props();

	const {
		elements: { trigger, overlay, content, portalled },
		states: { open }
	} = createDialog({
		forceVisible: true
	});
</script>

<button class="contents cursor-zoom-in" use:melt={$trigger}>
	{@render children()}
</button>

{#if $open}
	<div use:melt={$portalled}>
		<div
			class="fixed inset-0 z-50 bg-black/50"
			transition:fade={{ duration: 150 }}
			use:melt={$overlay}
		></div>

		<div
			class="fixed top-1/2 left-1/2 z-50 max-h-[85vh] -translate-x-1/2 -translate-y-1/2"
			transition:fade={{ duration: 150 }}
			use:melt={$content}
		>
			{@render children()}
		</div>
	</div>
{/if}
