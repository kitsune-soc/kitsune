<script lang="ts">
	import defaultAvatar from '$assets/default-avatar.png';
	import defaultHeader from '$assets/default-header.png';
	import Timeline from '$lib/components/Timeline.svelte';
	import Zoom from '$lib/components/Zoom.svelte';
	import type { Post } from '$lib/types/Post';

	import type { PageData } from './$houdini';
	import IconErrorOutline from '~icons/mdi/error-outline';

	let { data }: { data: PageData } = $props();

	let loadAccount = $derived(data.LoadAccount);
	let errors = $derived($loadAccount.errors);

	let account: {
		id: string;
		displayName?: string;
		username: string;
		headerUrl: string;
		avatarUrl: string;
	} = $derived({
		id: $loadAccount.data?.getAccountById?.id ?? '',
		displayName: $loadAccount.data?.getAccountById?.displayName ?? undefined,
		username: $loadAccount.data?.getAccountById?.username ?? '',
		headerUrl: $loadAccount.data?.getAccountById?.header?.url ?? defaultHeader,
		avatarUrl: $loadAccount.data?.getAccountById?.avatar?.url ?? defaultAvatar
	});

	let posts: Post[] = $derived(
		$loadAccount.data?.getAccountById?.posts.edges
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

		const result = await loadAccount.loadNextPage();
		reachedEnd = lastPostLength === result.data?.getAccountById?.posts.edges.length;
		lastPostLength = result.data?.getAccountById?.posts.edges.length ?? lastPostLength;

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
</script>

<main class="m-auto w-full max-w-prose">
	{#if errors}
		<div role="alert" class="alert alert-error shadow-xl">
			<IconErrorOutline />
			<span>
				<ul class="list-none p-0">
					{#each errors as error, idx (idx)}
						<li>{error.message}</li>
					{/each}
				</ul>
			</span>
		</div>
	{:else}
		<div class="card image-full w-full shadow-xl">
			<figure class="m-0">
				<img class="h-full w-full" src={account.headerUrl} alt="Header" />
			</figure>
			<div class="card-body flex justify-end">
				<div
					class="card bg-base-100 text-base-content flex max-w-1/2 flex-row items-end gap-3 p-3 shadow-xl"
				>
					<div class="avatar">
						<div class="w-24 rounded">
							<Zoom>
								<img class="m-0" src={account.avatarUrl} alt="Avatar" />
							</Zoom>
						</div>
					</div>

					<div class="flex flex-col">
						{account.displayName ?? account.username}
						<span class="font-bold">
							@{account.username}
						</span>
					</div>
				</div>
			</div>
		</div>

		<div class="mt-5">
			<Timeline {posts} {onendreached} />
		</div>
	{/if}
</main>
