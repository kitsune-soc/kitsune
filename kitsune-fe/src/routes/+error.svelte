<script lang="ts">
	import { page } from '$app/state';
	import Glitch from '$lib/components/Glitch.svelte';
</script>

<main class="flex min-h-screen w-screen flex-col place-content-center items-center">
	<Glitch>
		<div class="flex flex-col items-center text-3xl">
			<h1 class="m-0">
				{page.status}
			</h1>

			{page.error?.message}
		</div>
	</Glitch>

	<p>
		<a href="/">Back to the main page</a>
	</p>
</main>
