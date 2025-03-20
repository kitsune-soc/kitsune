<script lang="ts">
	import { page } from '$app/state';

	import type { Component } from 'svelte';

	let { name, icon: Icon, url }: { name: string; icon: Component; url: string } = $props();
</script>

<li>
	<a class="no-underline" class:menu-active={page.url.pathname === url} href={url}>
		<Icon />
		{name}
	</a>
</li>
