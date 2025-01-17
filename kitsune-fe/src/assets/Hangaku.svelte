<script lang="ts">
	import Glitch from '$lib/components/Glitch.svelte';

	import type { HTMLAttributes } from 'svelte/elements';

	import hangakuRaw from './hangaku.svg';

	let props: HTMLAttributes<HTMLDivElement> = $props();
</script>

<Glitch {...props}>
	<img class="m-0 h-auto w-full" src={hangakuRaw} alt="Hangaku (Kitsune mascot)" />
</Glitch>
