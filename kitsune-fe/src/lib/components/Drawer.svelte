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
				<li>
					<a href={data.url}>
						<Icon class="h-4 w-auto" icon={data.icon} />
						{name}
					</a>
				</li>
			{/each}

			<li>Settings</li>
			<li>
				<a href="/settings/account">
					<Icon class="h-4 w-auto" icon="mdi:account-settings" /> Account
				</a>
			</li>
			<li>
				<a href="/settings/frontend">
					<Icon class="h-4 w-auto" icon="mdi:settings" /> Frontend
				</a>
			</li>
			<li>
				<a href="/settings/admin">
					<Icon class="h-4 w-auto" icon="mdi:administrator" /> Administrator
				</a>
			</li>

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
