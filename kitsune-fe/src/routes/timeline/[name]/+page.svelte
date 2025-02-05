<script lang="ts">
	import { page } from '$app/state';
	import Timeline from '$lib/components/Timeline.svelte';

	const name = page.params.name;
</script>

<main class="min-h-screen w-full">
	<Timeline posts={[]} />
</main>
