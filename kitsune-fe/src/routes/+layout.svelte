<script lang="ts">
	import type { Snippet } from 'svelte';
	import '../styles/root.scss';
	import type { PageData } from './$houdini';
	import { version as frontendVersion } from '$app/environment';

	import '../app.css';

	const { children, data }: { children: Snippet; data: PageData } = $props();

	let backendVersionStore = $derived(data.version);
	let backendVersion = $derived($backendVersionStore.data?.instance.version ?? '[unknown]');
</script>

<svelte:head>
	<title>Kitsune Ⓐ🏴</title>
	<!-- Disable dark reader -->
	<meta name="darkreader-lock" />
</svelte:head>

{@render children()}

<footer class="text-sm max-lg:mb-5 max-lg:text-center lg:fixed lg:bottom-3 lg:left-3">
	<p>
		Backend version: {backendVersion}
		<br />Frontend version: {frontendVersion}
	</p>

	<span>
		Powered by
		<a target="_blank" href="https://github.com/kitsune-soc/kitsune">Kitsune</a>
	</span>
</footer>
