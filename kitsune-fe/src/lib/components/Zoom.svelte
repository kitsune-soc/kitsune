<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<!-- TODO: Impl zoom -->
<button class="contents cursor-zoom-in">
	{@render children()}
</button>
