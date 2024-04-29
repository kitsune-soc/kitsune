<script lang="ts">
	import { DEFAULT_PROFILE_PICTURE_URL } from '../consts';
	import NavBarLink from './NavBarLink.svelte';

	type RouteInfo = {
		icon: string;
		detail: string;
	};

	const links: Record<string, RouteInfo> = {
		'/timeline/home': {
			icon: 'fa-house fa-solid',
			detail: 'Home'
		},
		'/notifications': {
			icon: 'fa-bell fa-solid',
			detail: 'Notification'
		},
		'/messages': {
			icon: 'fa-envelope fa-solid',
			detail: 'Messages'
		},
		'/timeline/local': {
			icon: 'fa-users fa-solid',
			detail: 'Local'
		},
		'/timeline/federated': {
			icon: 'fa-globe-europe fa-solid',
			detail: 'Federated'
		}
	};

	//const NewPostModal = defineAsyncComponent(() => import('./modal/NewPostModal.vue'));
	let showPostModal = $state(false);
</script>

<nav class="nav-bar">
	<div class="nav-bar-links">
		{#each Object.entries(links) as [route, details]}
			<NavBarLink to={route} icon={details.icon} detail={details.detail} />
		{/each}
	</div>

	<div class="nav-bar-profile">
		<div class="nav-bar-element profile-menu-button">
			<img src={DEFAULT_PROFILE_PICTURE_URL} />
		</div>

		<div class="nav-bar-element">
			<button
				class="icon create-status fa-pen-to-square fa-solid"
				onclick={() => (showPostModal = true)}
			></button>
		</div>
	</div>
</nav>

<!--<NewPostModal v-model="showPostModal" />-->

<style lang="scss">
	@use '../../styles/colours' as *;
	@use '../../styles/mixins' as *;

	.nav-bar {
		display: flex;
		position: fixed;
		top: 0;
		right: 0;
		left: 0;
		justify-content: space-between;
		align-items: center;
		z-index: 999;
		margin-bottom: 100px;
		background-color: $dark2;
		padding: 0 25px;
		padding-top: 5px;

		@include only-on-mobile {
			padding: 0;
			padding-top: 5px;

			& .detail {
				display: none;
			}

			& .icon {
				margin-right: 0px;
			}
		}

		&-profile {
			display: flex;
			gap: 10px;

			.create-status {
				cursor: pointer;
				height: 25px;
			}

			.profile-menu-button {
				display: flex;
				align-items: center;
				border-radius: 4px;

				img {
					height: 30px;
				}
			}
		}
	}
</style>
