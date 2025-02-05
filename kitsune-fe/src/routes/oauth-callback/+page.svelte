<script lang="ts">
	import { page } from '$app/state';
	import { fetchOAuthToken } from '$lib/oauth/token.svelte';
	import Icon from '@iconify/svelte';

	async function loadAccessToken(oauthCode: string): Promise<void> {
		await fetchOAuthToken(oauthCode);
	}

	const oauthCode = page.url.searchParams.get('code');
	loadAccessToken(oauthCode!);
</script>

<main class="flex h-screen w-full place-items-center justify-center gap-5">
	<Icon icon="line-md:loading-loop" class="h-auto w-20" />
	Logging in..
</main>
