<script lang="ts">
	import type { Post } from '$lib/types/Post';
	import { createWindowVirtualizer } from '@tanstack/svelte-virtual';

	import PostComponent from './Post.svelte';

	let { posts }: { posts: Array<Post> } = $props();

	let timelineElement: HTMLDivElement | undefined = $state();

	let virtualizer = $derived(
		createWindowVirtualizer<HTMLDivElement>({
			count: posts.length,
			scrollMargin: timelineElement?.offsetTop ?? 0,
			estimateSize: () => 45
		})
	);

	let virtualItems = $derived($virtualizer.getVirtualItems());
	let virtualElements: HTMLDivElement[] = $state([]);

	$effect(() => {
		virtualElements.forEach((element) => $virtualizer.measureElement(element));
	});
</script>

<div bind:this={timelineElement} style="height: {$virtualizer.getTotalSize()}px;">
	<div
		style="transform: translateY({virtualItems[0]
			? virtualItems[0].start - $virtualizer.options.scrollMargin
			: 0}px);"
	>
		{#each virtualItems as row}
			<div bind:this={virtualElements[row.index]} data-index={row.index}>
				<PostComponent {...posts[row.index]} />
			</div>
		{/each}
	</div>
</div>
