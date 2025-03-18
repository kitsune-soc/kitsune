<script lang="ts">
	import { page } from '$app/state';
	import Hero from '$lib/components/Hero.svelte';

	let id = $derived(page.params.id);
</script>

<Hero>
	<h1>Account of {id}</h1>
</Hero>
