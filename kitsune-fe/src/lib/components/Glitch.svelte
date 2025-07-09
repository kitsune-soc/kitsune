<script lang="ts">
	import { PowerGlitch, type PlayModes } from 'powerglitch';
	import type { Snippet } from 'svelte';
	import type { HTMLAttributes } from 'svelte/elements';

	let {
		mode,
		children,
		...props
	}: { mode?: PlayModes; children: Snippet } & HTMLAttributes<HTMLDivElement> = $props();

	let wrapper: HTMLElement | undefined = $state();

	$effect(() => {
		if (!wrapper) {
			return;
		}

		PowerGlitch.glitch(wrapper, {
			playMode: mode,
			timing: { duration: 4_000, iterations: Infinity },
			shake: false
		});
	});
</script>

<div {...props}>
	<div bind:this={wrapper}>
		{@render children()}
	</div>
</div>
