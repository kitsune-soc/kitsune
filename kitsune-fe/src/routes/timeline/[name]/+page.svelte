<script lang="ts">
	//import { page } from '$app/state';
	import { LoadHomeTimelineStore } from '$houdini';
	import NewPost from '$lib/components/NewPost.svelte';
	import Timeline from '$lib/components/Timeline.svelte';
	import type { Post } from '$lib/types/Post';

	import type { PageData } from './$houdini';

	let { data }: { data: PageData } = $props();
	let characterLimitStore = $derived(data.LoadCharacterLimit);

	//const name = $derived(page.params.name);

	let homeTimeline = new LoadHomeTimelineStore();
	let posts: Post[] = $derived(
		$homeTimeline.data?.homeTimeline.edges
			.map((edge) => edge.node)
			.map((post) => {
				return {
					id: post.id,
					user: {
						id: post.account.id,
						name: post.account.displayName ?? post.account.username,
						username: post.account.username
					},
					content: post.content,
					attachments: post.attachments,
					replyCount: 0,
					likeCount: 0,
					repostCount: 0,
					url: post.url,
					createdAt: post.createdAt,
					visibility: post.visibility
				};
			}) ?? []
	);
	let lastPostLength = $state(0);

	let reachedEnd = $state(false);

	let timelineMeta: { loadingNewPosts: boolean } = $state({
		loadingNewPosts: false
	});

	async function loadTimeline() {
		console.log(`last post length before: ${lastPostLength}`);

		const result = await homeTimeline.loadNextPage();
		reachedEnd = lastPostLength === result.data?.homeTimeline.edges.length;
		lastPostLength = result.data?.homeTimeline.edges.length ?? lastPostLength;

		console.log(`reached end: ${reachedEnd}`);
		console.log(`last post length after: ${lastPostLength}`);
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

	function onnewpost() {
		homeTimeline.loadPreviousPage();
	}

	// initial timeline load
	homeTimeline.fetch();
</script>

<main class="m-auto max-w-prose">
	<NewPost characterLimit={$characterLimitStore?.data?.instance.characterLimit ?? 0} {onnewpost} />
	<div class="divider"></div>
	<Timeline {posts} {onendreached} />
</main>
