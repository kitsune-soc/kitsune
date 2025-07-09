<script lang="ts">
	import { onNavigate } from '$app/navigation';

	import type { Snippet } from 'svelte';

	import Navbar from '../Navbar.svelte';
	import DrawerContent from './DrawerContent.svelte';

	let { children }: { children: Snippet } = $props();

	let drawerItemName = 'drawer-toggle';
	let drawerToggle: HTMLInputElement | undefined = $state();

	onNavigate(() => {
		if (drawerToggle) {
			drawerToggle.checked = false;
		}
	});
</script>

<div class="drawer">
	<input bind:this={drawerToggle} id={drawerItemName} type="checkbox" class="drawer-toggle" />
	<div class="drawer-content flex flex-col">
		<Navbar {drawerItemName} />
		<div>
			{@render children()}
		</div>
	</div>

	<div class="drawer-side z-20">
		<label for={drawerItemName} aria-label="close sidebar" class="drawer-overlay"> </label>
		<DrawerContent />
	</div>
</div>
