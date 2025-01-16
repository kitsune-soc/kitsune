<script lang="ts">
	import { version as frontendVersion } from '$app/environment';
	import { i18n } from '$lib/i18n';
	import { ParaglideJS } from '@inlang/paraglide-sveltekit';

	import type { Snippet } from 'svelte';

	import '../app.css';
	import type { PageData } from './$houdini';

	const { children, data }: { children: Snippet; data: PageData } = $props();

	let backendVersionStore = $derived(data.version);
	let backendVersion = $derived($backendVersionStore.data?.instance.version ?? '[unknown]');
</script>

<svelte:head>
	<title>Kitsune Ⓐ🏴</title>
	<!-- Disable dark reader -->
	<meta name="darkreader-lock" />
</svelte:head>

<ParaglideJS {i18n}>
	{@render children()}

	<footer class="w-full text-sm max-lg:mb-5 max-lg:text-center lg:fixed lg:bottom-3 lg:left-3">
		<p>
			Backend version: {backendVersion}
			<br />Frontend version: {frontendVersion}
		</p>

		<span>
			Powered by
			<a target="_blank" href="https://github.com/kitsune-soc/kitsune">Kitsune</a>
		</span>
	</footer>
</ParaglideJS>
