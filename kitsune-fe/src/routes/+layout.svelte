<script lang="ts">
	import '../styles/root.scss';

	import NavBar from '$lib/components/NavBar.svelte';

	let { children } = $props();
</script>

<svlete:head>
	<title>Kitsune</title>
</svlete:head>

<NavBar />

{@render children()}
