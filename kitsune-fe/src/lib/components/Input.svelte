<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';

	let props: HTMLInputAttributes = $props();
</script>

<input
	{...props}
	class="w-full border-0 border-b-2 border-gray-200 bg-transparent disabled:cursor-not-allowed {props.class}"
/>
