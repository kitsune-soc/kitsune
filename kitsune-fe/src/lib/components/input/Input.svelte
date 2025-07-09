<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';

	let props: HTMLInputAttributes = $props();
</script>

<input {...props} class="input validator w-full {props.class}" />
