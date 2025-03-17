<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { fetchOAuthToken } from '$lib/oauth/token';

	const oauthCode = page.url.searchParams.get('code');
	fetchOAuthToken(oauthCode!).then(() => goto('/'));
</script>

<main class="hero min-h-screen">
	<div class="hero-content">
		<span class="loading loading-spinner loading-xl"></span>
		Logging in..
	</div>
</main>
