<script lang="ts">
	import defaultAvatar from '$assets/default-avatar.png';
	import defaultHeader from '$assets/default-header.png';
	import type { FullUser } from '$lib/types/User';

	import { DateTime } from 'luxon';

	let {
		name,
		username,
		description,
		avatarUrl = defaultAvatar,
		headerUrl = defaultHeader,
		createdAt: rawCreatedAt
	}: FullUser = $props();

	const parsedCreatedAt = new Date(rawCreatedAt);

	function formatRelative(date: Date): string {
		return DateTime.fromJSDate(date).toRelative()!;
	}

	let createdAt = $state(formatRelative(parsedCreatedAt));

	$effect(() => {
		const interval = setInterval(() => {
			createdAt = formatRelative(parsedCreatedAt);
		}, 5_000);

		return () => clearInterval(interval);
	});
</script>

<div class="max-w-96 rounded border-2 border-gray-200 bg-dark-1">
	<figure class="relative m-0">
		<div class="h-44 bg-cover opacity-50" style="background-image: url({headerUrl})"></div>

		<figcaption class="absolute bottom-0 left-0 m-2 flex flex-row place-items-end gap-3">
			<img class="m-0 w-20 rounded" src={avatarUrl} alt="{name} avatar" />

			<div class="text-white">
				<h3 class="m-0">{name}</h3>
				{username}
			</div>
		</figcaption>
	</figure>

	<p class="m-3 leading-snug">{description}</p>
</div>
