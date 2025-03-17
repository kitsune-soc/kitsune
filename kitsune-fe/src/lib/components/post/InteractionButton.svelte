<script lang="ts">
	import type { Component } from 'svelte';

	let { icon: Icon, count, oninteract }: { icon: Component; count?: number; oninteract?: () => void } =
		$props();
</script>

<button
	class="flex cursor-pointer flex-row gap-1.5"
	onclick={() => {
		if (oninteract) oninteract();
	}}
>
	<Icon />

	{#if count && count > 0}
		{count}
	{/if}
</button>
