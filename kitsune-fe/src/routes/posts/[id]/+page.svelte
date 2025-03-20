<script lang="ts">
	import { Post } from '$lib/components/post';

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

<main class="flex flex-col items-center">
	{#if post && user}
		<div class="w-full max-w-prose">
			<Post
				id={post.id}
				{user}
				content={post.content}
				attachments={post.attachments}
				visibility={post.visibility}
				url={post.url}
				createdAt={post.createdAt}
				likeCount={0}
				replyCount={0}
				repostCount={0}
			/>
		</div>
	{/if}
</main>
