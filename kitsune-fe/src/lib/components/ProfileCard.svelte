<script lang="ts">
	import defaultAvatar from '$assets/default-avatar.png';
	import defaultHeader from '$assets/default-header.png';
	import type { FullUser } from '$lib/types/User';

	let {
		name,
		username,
		description,
		avatarUrl = defaultAvatar,
		headerUrl = defaultHeader
	}: FullUser = $props();
</script>

<div class="bg-dark-1 max-w-96 rounded border-2 border-gray-200">
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

	<p class="m-3">{description}</p>
</div>
