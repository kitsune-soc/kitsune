<script lang="ts">
	import defaultAvatar from '$assets/default-avatar.png';
	import defaultHeader from '$assets/default-header.png';
	import Zoom from '$lib/components/Zoom.svelte';

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
					class="card bg-base-100 text-base-content join join-horizontal max-w-1/2 items-end gap-3 p-3 shadow-xl"
				>
					<div class="avatar">
						<div class="w-24 rounded">
							<Zoom>
								<img class="m-0" src={account.avatarUrl} alt="Avatar" />
							</Zoom>
						</div>
					</div>

					<div class="join join-vertical">
						{account.displayName ?? account.username}
						<span class="font-bold">
							@{account.username}
						</span>
					</div>
				</div>
			</div>
		</div>
		<p>And here we display all the posts. Yep.</p>
	{/if}
</main>
