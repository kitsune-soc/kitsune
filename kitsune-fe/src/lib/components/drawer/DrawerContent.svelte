<script lang="ts">
	import { goto } from '$app/navigation';
	import { clearTokenStorage } from '$lib/oauth/token';

	import type { Component } from 'svelte';

	import CyberpunkMode from '../CyberpunkMode.svelte';
	import DrawerItem from './DrawerItem.svelte';
	import IconAccountSettings from '~icons/mdi/account-settings';
	import IconAdministrator from '~icons/mdi/administrator';
	import IconGlobe from '~icons/mdi/globe';
	import IconHome from '~icons/mdi/home';
	import IconLogout from '~icons/mdi/logout';
	import IconPeople from '~icons/mdi/people';
	import IconSettings from '~icons/mdi/settings';

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

<ul class="menu bg-base-200 m-0 min-h-full w-60 p-4">
	<li>Timelines</li>
	{#each Object.entries(timelines) as [name, data] (name)}
		<DrawerItem {name} icon={data.icon} url={data.url} />
	{/each}

	<li>Settings</li>

	<DrawerItem name="Account" icon={IconAccountSettings} url="/settings/account" />
	<DrawerItem name="Frontend" icon={IconSettings} url="/settings/frontend" />
	<DrawerItem name="Administrator" icon={IconAdministrator} url="/settings/admin" />

	<div class="divider"></div>

	<CyberpunkMode />

	<div class="divider"></div>

	<li>
		<button class="btn" onclick={logout}>
			<IconLogout /> Logout
		</button>
	</li>
</ul>
