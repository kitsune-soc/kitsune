<script lang="ts">
	import type { Post } from '$lib/types/Post';

	import InteractionButton from './InteractionButton.svelte';
	import RelativeTime from './RelativeTime.svelte';

	let {
		id,
		user,
		content,
		createdAt,
		replyCount,
		likeCount,
		repostCount,
		primary = true
	}: Post & { primary?: boolean } = $props();

	let postLink = $derived(`/posts/${id}`);
</script>

{#snippet renderPost()}
	<article class="flex w-full flex-row p-3">
		<div class="w-16">
			<img class="m-0 h-auto w-full rounded" src={user.avatarUrl} alt="{user.username} avatar" />
		</div>

		<div class="ml-3 w-full">
			<div>
				<strong>{user.name}</strong>

				<a class="text-shade2-light break-keep no-underline" href="/users/{user.id}">
					@{user.username}
				</a>
			</div>

			<div class="my-2">
				<!-- eslint-disable-next-line svelte/no-at-html-tags -->
				{@html content}
			</div>

			<div class="flex flex-row justify-between">
				<InteractionButton icon="material-symbols:reply-rounded" count={replyCount} />
				<InteractionButton icon="material-symbols:repeat-rounded" count={repostCount} />
				<InteractionButton icon="material-symbols:star-rounded" count={likeCount} />

				<InteractionButton icon="material-symbols:menu-rounded" />

				<a class="no-underline hover:underline" href={postLink}>
					<RelativeTime time={createdAt} />
				</a>
			</div>
		</div>
	</article>
{/snippet}

{#if primary}
	{@render renderPost()}
{:else}
	<a href={postLink} class="no-underline">
		{@render renderPost()}
	</a>
{/if}
