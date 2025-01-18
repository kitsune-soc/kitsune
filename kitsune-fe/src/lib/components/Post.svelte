<script lang="ts">
	import type { Post } from '$lib/types/Post';

	import InteractionButton from './InteractionButton.svelte';
	import RelativeTime from './RelativeTime.svelte';

	let { id, user, content, createdAt, replyCount, likeCount, repostCount }: Post = $props();
</script>

<article class="flex w-[40vw] flex-row p-3">
	<div class="w-16">
		<img class="m-0 h-auto w-full rounded" src={user.avatarUrl} alt="{user.username} avatar" />
	</div>

	<div class="ml-3 w-full">
		<div>
			<strong>{user.name}</strong>

			<a class="break-keep text-shade2-light no-underline" href="/users/{user.id}">
				{user.username}
			</a>
		</div>

		<div class="my-2">
			{content}
		</div>

		<div class="flex flex-row justify-between">
			<InteractionButton icon="material-symbols:reply-rounded" count={replyCount} />
			<InteractionButton icon="material-symbols:repeat-rounded" count={repostCount} />
			<InteractionButton icon="material-symbols:star-rounded" count={likeCount} />
			<InteractionButton icon="material-symbols:menu-rounded" />
			<a class="no-underline hover:underline" href="/posts/{id}">
				<RelativeTime time={createdAt} />
			</a>
		</div>
	</div>
</article>
