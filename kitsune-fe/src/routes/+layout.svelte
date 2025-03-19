<script lang="ts">
	import { version as frontendVersion } from '$app/environment';
	import Footer from '$lib/components/Footer.svelte';
	import { Drawer } from '$lib/components/drawer';

	import type { Snippet } from 'svelte';
	import { pwaInfo } from 'virtual:pwa-info';

	import '../app.css';
	import type { PageData } from './$houdini';

	const { children, data }: { children: Snippet; data: PageData } = $props();

	let backendVersionStore = $derived(data.LoadVersion);
	let backendVersion = $derived($backendVersionStore.data?.instance.version ?? '[unknown]');

	let webManifestLink = $derived(pwaInfo ? pwaInfo.webManifest.linkTag : '');
</script>

<svelte:head>
	<title>Kitsune Ⓐ🏴</title>
	<!-- Disable dark reader -->
	<meta name="darkreader-lock" />

	<!-- eslint-disable-next-line svelte/no-at-html-tags -->
	{@html webManifestLink}
</svelte:head>

<Drawer>
	{@render children()}
	<Footer {backendVersion} {frontendVersion} />
</Drawer>
