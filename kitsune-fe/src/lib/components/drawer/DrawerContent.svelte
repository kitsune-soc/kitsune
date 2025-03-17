<script lang="ts">
	import { goto } from '$app/navigation';

	import IconAdministrator from '~icons/mdi/administrator';
	import IconAccountSettings from '~icons/mdi/account-settings';
	import IconSettings from '~icons/mdi/settings';
	import IconHome from '~icons/mdi/home';
	import IconPeople from '~icons/mdi/people';
	import IconGlobe from '~icons/mdi/globe';
	import IconLogout from '~icons/mdi/logout';

	import { clearTokenStorage } from '$lib/oauth/token';
	import type { Component } from 'svelte';

	let timelines: Record<string, { icon: Component; url: string }> = {
		Home: {
			icon: IconHome,
			url: '/timeline/home'
		},
		Local: {
			icon: IconPeople,
			url: '/timeline/local'
		},
		Global: {
			icon: IconGlobe,
			url: '/timeline/global'
		}
	};

	function logout() {
		clearTokenStorage();
		goto('/');
	}
</script>

{#snippet sidebarItem(name: string, Icon: Component, url: string)}
	<li>
		<a href={url}>
			<Icon />
			{name}
		</a>
	</li>
{/snippet}

<ul class="menu bg-base-200 min-h-full w-80 p-4">
	<li>Timelines</li>
	{#each Object.entries(timelines) as [name, data]}
		{@render sidebarItem(name, data.icon, data.url)}
	{/each}

	<li>Settings</li>
	{@render sidebarItem('Account', IconAccountSettings, '/settings/account')}
	{@render sidebarItem('Frontend', IconSettings, '/settings/frontend')}
	{@render sidebarItem('Administrator', IconAdministrator, '/settings/admin')}

	<div class="divider"></div>

	<li>
		<button class="btn" onclick={logout}>
			<IconLogout /> Logout
		</button>
	</li>
</ul>

<style>
	a {
		@apply no-underline;
	}
</style>
