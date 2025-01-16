<script lang="ts">
	import { PowerGlitch } from 'powerglitch';
	import type { HTMLAttributes } from 'svelte/elements';

	import hangakuRaw from './hangaku.svg';

	let props: HTMLAttributes<HTMLDivElement> = $props();
	let hangakuElement: HTMLImageElement | undefined = $state();

	$effect(() => {
		PowerGlitch.glitch(hangakuElement, {
			playMode: 'hover'
		});
	});
</script>

<div {...props}>
	<img
		class="m-0 h-auto w-full"
		bind:this={hangakuElement}
		src={hangakuRaw}
		alt="Hangaku (Kitsune mascot)"
	/>
</div>
