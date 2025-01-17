<script lang="ts">
	import { SvelteDate } from 'svelte/reactivity';

	let { content, createdAt }: { content: string; createdAt: SvelteDate } = $props();
</script>
