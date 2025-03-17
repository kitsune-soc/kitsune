<script lang="ts">
	import type { Snippet } from 'svelte';

	import Navbar from '../Navbar.svelte';
	import DrawerContent from './DrawerContent.svelte';

	let { children }: { children: Snippet } = $props();

	let drawerItemName = 'drawer-toggle';
</script>

<div class="drawer">
	<input id={drawerItemName} type="checkbox" class="drawer-toggle" />
	<div class="drawer-content flex flex-col">
		<Navbar {drawerItemName} />
		{@render children()}
	</div>

	<div class="drawer-side">
		<label for={drawerItemName} aria-label="close sidebar" class="drawer-overlay"> </label>
		<DrawerContent />
	</div>
</div>
