<script lang="ts">
	import { goto } from '$app/navigation';
	import { clearTokenStorage } from '$lib/oauth/token';
	import Icon from '@iconify/svelte';

	import type { Snippet } from 'svelte';

	import Navbar from './Navbar.svelte';

	let { children }: { children: Snippet } = $props();

	let drawerItemName = 'drawer-toggle';
	let timelines: Record<string, { icon: string; url: string }> = {
		Home: {
			icon: 'mdi:home',
			url: '/timeline/home'
		},
		Local: {
			icon: 'mdi:people',
			url: '/timeline/local'
		},
		Global: {
			icon: 'mdi:globe',
			url: '/timeline/global'
		}
	};

	function logout() {
		clearTokenStorage();
		goto('/');
	}
</script>

{#snippet sidebarItem(name: string, icon: string, url: string)}
	<li>
		<a href={url}>
			<Icon class="h-4 w-auto" {icon} />
			{name}
		</a>
	</li>
{/snippet}

<div class="drawer">
	<input id={drawerItemName} type="checkbox" class="drawer-toggle" />
	<div class="drawer-content flex flex-col">
		<Navbar {drawerItemName} />

		{@render children()}
	</div>

	<div class="drawer-side">
		<label for={drawerItemName} aria-label="close sidebar" class="drawer-overlay"> </label>
		<ul class="menu bg-base-200 min-h-full w-80 p-4">
			<!-- Sidebar content here -->
			<li>Timelines</li>
			{#each Object.entries(timelines) as [name, data]}
				{@render sidebarItem(name, data.icon, data.url)}
			{/each}

			<li>Settings</li>
			{@render sidebarItem('Account', 'mdi:account-settings', '/settings/account')}
			{@render sidebarItem('Frontend', 'mdi:settings', '/settings/frontend')}
			{@render sidebarItem('Administrator', 'mdi:administrator', '/settings/admin')}

			<div class="divider"></div>

			<li>
				<button class="btn" onclick={logout}>
					<Icon class="h-4 w-auto" icon="mdi:logout" /> Logout
				</button>
			</li>
		</ul>
	</div>
</div>

<style>
	a {
		@apply no-underline;
	}
</style>
