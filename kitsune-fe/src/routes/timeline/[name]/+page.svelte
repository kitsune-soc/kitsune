<script lang="ts">
	//import { page } from '$app/state';
	import { LoadHomeTimelineStore } from '$houdini';
	import Timeline from '$lib/components/Timeline.svelte';
	import type { Post } from '$lib/types/Post';

	//const name = $derived(page.params.name);

	let homeTimeline = new LoadHomeTimelineStore();
	let posts: Post[] = $state([]);
	let reachedEnd = $state(false);

	let timelineMeta: { loadingNewPosts: boolean; after?: string } = $state({
		loadingNewPosts: false,
		after: undefined
	});

	async function loadTimeline() {
		const result = await homeTimeline.fetch({
			variables: { after: timelineMeta.after }
		});

		const mappedPosts =
			result.data?.homeTimeline.nodes.map((post): Post => {
				return {
					id: post.id,
					user: {
						id: post.account.id,
						name: post.account.displayName ?? post.account.username,
						username: post.account.username
					},
					content: post.content,
					replyCount: 0,
					likeCount: 0,
					repostCount: 0,
					url: post.url,
					createdAt: post.createdAt,
					visibility: post.visibility
				};
			}) ?? [];

		reachedEnd = mappedPosts.length === 0;

		posts = posts.concat(mappedPosts);
		timelineMeta.after = result.data?.homeTimeline.pageInfo.endCursor ?? undefined;
	}

	async function onendreached() {
		if (reachedEnd) return;
		if (timelineMeta.loadingNewPosts) return;

		timelineMeta.loadingNewPosts = true;

		try {
			await loadTimeline();
		} catch (error) {
			console.error(`failed to load posts: ${error}`);
		}

		timelineMeta.loadingNewPosts = false;
	}

	// initial timeline load
	loadTimeline();
</script>

<main class="m-auto mt-18 max-w-prose">
	<Timeline {posts} {onendreached} />
</main>
