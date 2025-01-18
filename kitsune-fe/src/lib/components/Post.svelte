<script lang="ts">
	import type { Post } from '$lib/types/Post';

	import InteractionButton from './InteractionButton.svelte';
	import RelativeTime from './RelativeTime.svelte';

	let { id, user, content, createdAt, replyCount, likeCount, repostCount }: Post = $props();
</script>

<article class="flex w-[40vw] flex-row border-y border-gray-200 p-2">
	<div class="w-16">
		<img class="m-0 h-auto w-full" src={user.avatarUrl} alt="{user.username} avatar" />
	</div>

	<div class="ml-3 w-full">
		<div class="flex flex-row justify-between">
			<div class="my-1.5 leading-snug">
				<strong>{user.name}</strong>

				<a class="break-keep text-shade2-light no-underline" href="/users/{user.id}">
					{user.username}
				</a>
			</div>

			<a class="whitespace-nowrap no-underline hover:underline" href="/posts/{id}">
				<RelativeTime time={createdAt} />
			</a>
		</div>

		{content}

		<div class="mt-2 flex flex-row justify-between pr-20">
			<InteractionButton icon="material-symbols:reply-rounded" count={replyCount} />
			<InteractionButton icon="material-symbols:repeat-rounded" count={repostCount} />
			<InteractionButton icon="material-symbols:star-rounded" count={likeCount} />
			<InteractionButton icon="material-symbols:menu-rounded" />
		</div>
	</div>
</article>
