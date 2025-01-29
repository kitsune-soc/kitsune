<script lang="ts">
	import { PendingValue } from '$houdini';
	import Post from '$lib/components/Post.svelte';

	import type { PageData } from './$houdini';

	let { data }: { data: PageData } = $props();

	let postStore = $derived(data.LoadPost);

	let post = $derived($postStore.data?.getPostById);
	let user = $derived.by(() => {
		if (!post || post === PendingValue) return undefined;

		return {
			id: post.account.id,
			name: post.account.displayName ?? post.account.username,
			username: post.account.username
		};
	});
</script>

<main class="flex min-h-screen w-screen place-items-center justify-center">
	{#if post && user && post !== PendingValue}
		<div class="rounded-md border border-gray-200">
			<Post
				id={post.id}
				{user}
				content={post.content}
				createdAt={post.createdAt}
				likeCount={0}
				replyCount={0}
				repostCount={0}
			/>
		</div>
	{/if}
</main>
