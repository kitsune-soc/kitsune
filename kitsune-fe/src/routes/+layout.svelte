<script lang="ts">
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

<footer>
	<p class="version">
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

	footer {
		font-size: small;
		line-height: normal;
	}

	@include only-on-mobile {
		footer {
			margin: 3em 0;
			text-align: center;
		}
	}

	@include not-on-mobile {
		footer {
			position: absolute;
			bottom: 0;
			left: 0;
			padding: 1em;
		}
	}
</style>
