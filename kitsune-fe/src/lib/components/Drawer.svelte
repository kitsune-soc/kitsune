<script lang="ts">
	import Navbar from './Navbar.svelte';
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();

	let drawerItemName = "drawer-toggle";
</script>

<div class="drawer">
	<input id={drawerItemName} type="checkbox" class="drawer-toggle" />
	<div class="drawer-content flex flex-col">
		<!-- Navbar -->
		<Navbar {drawerItemName} />

        <div class="mt-18">
            {@render children()}
        </div>
	</div>

	<div class="drawer-side">
		<label for={drawerItemName} aria-label="close sidebar" class="drawer-overlay">
		</label>
		<ul class="menu bg-base-200 min-h-full w-80 p-4">
			<!-- Sidebar content here -->
			<li>Pisser</li>
			<li><button>wow</button></li>
			<li><a>Sidebar Item 2</a></li>
		</ul>
	</div>
</div>
