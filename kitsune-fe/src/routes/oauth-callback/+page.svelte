<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import Hero from '$lib/components/Hero.svelte';
	import { fetchOAuthToken } from '$lib/oauth/token';

	const oauthCode = page.url.searchParams.get('code');
	fetchOAuthToken(oauthCode!).then(() => goto('/'));
</script>

<Hero>
	<span class="loading loading-spinner loading-xl"></span>
	Logging in..
</Hero>
