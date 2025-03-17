<script lang="ts">
	import IconMenu from '~icons/mdi/menu';

	let { drawerItemName }: { drawerItemName: string } = $props();
</script>

<nav class="navbar bg-base-300/50 not-prose fixed z-10 w-full backdrop-blur-sm">
	<div class="flex-none">
		<label for={drawerItemName} aria-label="open sidebar" class="btn btn-square btn-ghost">
			<IconMenu class="h-6" />
		</label>
	</div>
	<div class="mx-2 flex-1 px-2 text-3xl font-bold">Kitsune</div>
	<div class="hidden flex-none lg:block">
		<ul class="menu menu-horizontal"></ul>
	</div>
</nav>
