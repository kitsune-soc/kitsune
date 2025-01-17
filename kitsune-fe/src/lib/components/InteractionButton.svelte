<script lang="ts">
	import Icon from '@iconify/svelte';

	let { icon, count, oninteract }: { icon: string; count?: number; oninteract?: () => void } =
		$props();
</script>

<button
	class="flex flex-row gap-1.5"
	onclick={() => {
		if (oninteract) oninteract();
	}}
>
	<Icon {icon} class="h-auto w-5" />

	{#if count && count > 0}
		{count}
	{/if}
</button>
