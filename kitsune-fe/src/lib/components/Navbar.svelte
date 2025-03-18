<script lang="ts">
	import IconMenu from '~icons/mdi/menu';

	let { drawerItemName }: { drawerItemName: string } = $props();
</script>

<nav
	class="navbar bg-base-300/50 not-prose sticky top-0 z-10 mb-5 h-16 w-full shadow-sm backdrop-blur-sm"
>
	<div class="navbar-start">
		<label for={drawerItemName} aria-label="open sidebar" class="btn btn-square btn-ghost">
			<IconMenu class="h-6" />
		</label>
	</div>
	<div class="navbar-center">
		<div class="text-3xl font-bold">Kitsune</div>
	</div>
	<div class="navbar-end"></div>
</nav>
