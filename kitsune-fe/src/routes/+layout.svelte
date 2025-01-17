<script lang="ts">
	import { version as frontendVersion } from '$app/environment';
	import Footer from '$lib/components/Footer.svelte';

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

{@render children()}

<Footer {backendVersion} {frontendVersion} />
