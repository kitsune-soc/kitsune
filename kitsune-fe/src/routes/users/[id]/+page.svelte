<script lang="ts">
	import { page } from '$app/state';
	import Hero from '$lib/components/Hero.svelte';

	import type { PageData } from './$houdini';
	import IconErrorOutline from '~icons/mdi/error-outline';

	let id = $derived(page.params.id);
	let { data }: { data: PageData } = $props();

	let loadAccount = $derived(data.LoadAccount);

	let errors = $derived($loadAccount.errors);
	let account: {
		id: string;
		displayName: string;
		username: string;
		headerUrl: string;
		avatarUrl: string;
	} = $derived({
		id: $loadAccount.data?.getAccountById?.id ?? '',
		displayName: $loadAccount.data?.getAccountById?.displayName ?? '',
		username: $loadAccount.data?.getAccountById?.username ?? '',
		headerUrl: $loadAccount.data?.getAccountById?.header?.url ?? '',
		avatarUrl: $loadAccount.data?.getAccountById?.avatar?.url ?? ''
	});
</script>

<main class="m-auto">
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
		<h1>
			Account of "{account.displayName}" (Username: {account.username})

			<img src={account.headerUrl} alt="owo" />
			<img src={account.avatarUrl} alt="uwu" />

			<p class="text-center">
				Your ID is: <span class="card bg-base-100 p-5 shadow">{account.id}</span>
			</p>
		</h1>
	{/if}
</main>
