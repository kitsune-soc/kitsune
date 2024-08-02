<script lang="ts">
	import '../styles/root.scss';

	const { children } = $props();
</script>

<svelte:head>
	<title>Kitsune Ⓐ🏴</title>
	<!-- Disable dark reader -->
	<meta name="darkreader-lock" />
</svelte:head>

{@render children()}

<footer>
	Powered by
	<a target="_blank" href="https://github.com/kitsune-soc/kitsune">Kitsune</a>

	<p>
		Backend version: 0.0.1
		<br />Frontend version: 0.0.1
	</p>
</footer>

<style lang="scss">
	@use '../styles/breakpoints' as *;

	footer {
		position: fixed;

		bottom: 0;
		left: 0;
		padding: 1em;

		font-size: small;
		line-height: normal;
	}
</style>
