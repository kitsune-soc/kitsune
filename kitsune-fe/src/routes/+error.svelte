<script lang="ts">
	import { page } from '$app/state';
	import Glitch from '$lib/components/Glitch.svelte';
	import Hero from '$lib/components/Hero.svelte';
</script>

<Hero class="flex-col">
	<Glitch>
		<div class="flex flex-col items-center text-3xl">
			<h1 class="m-0">
				{page.status}
			</h1>

			{page.error?.message}
		</div>
	</Glitch>

	<a href="/">Back to the main page</a>
</Hero>
