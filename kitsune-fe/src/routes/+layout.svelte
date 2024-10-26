<script lang="ts">
	import '../app.css';
	import type { Snippet } from 'svelte';
	import '../styles/root.scss';
	import type { PageData } from './$houdini';
	import { version as frontendVersion } from '$app/environment';

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

<footer class="absolute bottom-3 left-3 text-sm">
	<p>
		Backend version: {backendVersion}
		<br />Frontend version: {frontendVersion}
	</p>

	<span>
		Powered by
		<a target="_blank" href="https://github.com/kitsune-soc/kitsune">Kitsune</a>
	</span>
</footer>

<style lang="scss">
	@use '../styles/mixins' as *;
</style>
