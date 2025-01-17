<script lang="ts">
	import type { User } from '$lib/types/User';
	import Icon from '@iconify/svelte';

	import InteractionButton from './InteractionButton.svelte';
	import RelativeTime from './RelativeTime.svelte';

	let {
		id,
		user,
		content,
		createdAt
	}: { id: string; user: User; content: string; createdAt: Date | string } = $props();
</script>

<article class="flex w-[40vw] flex-row border-y border-gray-200 p-2">
	<div>
		<img class="m-0 h-auto w-16" src={user.avatarUrl} alt="{user.username} avatar" />
	</div>

	<div class="ml-3 grow">
		<div class="flex flex-row justify-between">
			<div class="flex flex-row gap-2 leading-snug">
				<strong>{user.name}</strong>

				<a class="text-shade2-light no-underline" href="/users/{user.id}">
					{user.username}
				</a>
			</div>

			<a class="no-underline hover:underline" href="/posts/{id}">
				<RelativeTime time={createdAt} />
			</a>
		</div>

		{content}

		<div class="mt-2 flex flex-row justify-between pr-20">
			<InteractionButton icon="material-symbols:reply-rounded" count={3} />
			<InteractionButton icon="material-symbols:repeat-rounded" count={4} />
			<InteractionButton icon="material-symbols:star-rounded" count={12} />
			<InteractionButton icon="material-symbols:menu-rounded" />
		</div>
	</div>
</article>
