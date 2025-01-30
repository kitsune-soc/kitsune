<script lang="ts">
	import Post from '$lib/components/Post.svelte';

	import type { PageData } from './$houdini';

	let { data }: { data: PageData } = $props();

	let postStore = $derived(data.LoadPost);

	let post = $derived($postStore.data?.getPostById);
	let user = $derived.by(() => {
		if (!post) return undefined;

		return {
			id: post.account.id,
			name: post.account.displayName ?? post.account.username,
			username: post.account.username
		};
	});
</script>

<main class="flex min-h-screen w-screen place-items-center justify-center">
	{#if post && user}
		<div class="w-full max-w-prose rounded-md border border-gray-200">
			<Post
				id={post.id}
				{user}
				content={post.content}
				url={post.url}
				createdAt={post.createdAt}
				likeCount={0}
				replyCount={0}
				repostCount={0}
			/>
		</div>
	{/if}
</main>
