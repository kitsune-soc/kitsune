<script lang="ts">
	import Button from '$lib/components/Button.svelte';
	import type { PageData } from './$houdini';

	const { data }: { data: PageData } = $props();

	let statsStore = $derived(data.stats);
	let stats = $derived({
		postCount: $statsStore.data?.instance.localPostCount ?? 0,
		registeredUsers: $statsStore.data?.instance.userCount ?? 0
	});

	let registerButtonDisabled = $state(false);

	function initiateLogin() {
		alert('logging in wwowowowowowo');
	}
</script>

<div class="landing-page">
	<div class="section-left">
		<div class="section-left-content">
			<img class="logo" src="/kitsune_full.svg" alt="Kitsune logo" />

			<h1>Federated microblogging</h1>

			Statistics:

			<ul>
				<li>
					<strong>{stats.registeredUsers}</strong> registered users
				</li>
				<li>
					<strong>{stats.postCount}</strong> posts
				</li>
			</ul>
		</div>
	</div>

	<div class="section-right">
		<div class="section-right-content">
			<form class="register-form" method="post">
				<label for="username">Username</label>
				<input placeholder="hangaku" type="text" name="username" />

				<label for="email">Email address</label>
				<input placeholder="hangaku@joinkitsune.org" type="email" name="email" />

				<label for="password">Password</label>
				<input type="password" name="password" />

				<label for="confirm-password">Confirm Password</label>
				<input type="password" name="confirm-password" />

				<p>
					<Button class="register-button" disabled={registerButtonDisabled}>Register</Button>
				</p>
			</form>

			<Button buttonType="secondary" class="sign-in-button" onclick={initiateLogin}>
				Already have an account? Sign in
			</Button>
		</div>
	</div>
</div>

<style lang="scss">
	@use '../styles/breakpoints' as *;
	@use '../styles/colours' as *;
	@use '../styles/mixins' as *;
	@use 'sass:map';

	.landing-page {
		display: flex;
		flex-direction: row;
		height: 100%;
		width: 100%;

		.section-right {
			display: flex;
			flex-direction: column;
			justify-content: center;
			height: 100%;
			width: 100%;
			background-color: $dark2;

			&-content {
				border-radius: 0px 30px 30px 0px;
				background-color: $dark1;
				padding: 2em;
				max-width: 50ch;
				width: 100%;

				& :global(.sign-in-button) {
					width: 100%;
				}
			}
		}

		.section-left {
			display: flex;
			justify-content: center;
			align-items: center;
			height: 100%;
			width: 100%;
		}
	}

	.logo {
		width: 65%;
	}

	.register-form {
		display: flex;
		flex-direction: column;

		& label {
			margin-top: 0.5em;
		}

		& input {
			width: 100%;
			border: none;
			height: 40px;
			border-radius: 10px;
			background-color: $dark2;
			margin-bottom: 0.75em;
			padding-left: 1em;
		}

		& :global(.register-button) {
			width: 100%;
			margin-top: 1.5em;
		}
	}

	h1,
	ul {
		color: $text1;
	}

	@include only-on-mobile {
		.section-left {
			display: flex;
			flex-direction: column;
			align-items: center;
		}

		.landing-page {
			display: flex;
			flex-direction: column;

			.section-right {
				flex-direction: row;
				justify-content: center;

				&-content {
					max-width: 100%;
					border-radius: 0px;
					background-color: $dark2;
				}
			}
		}

		.register-form input {
			background-color: $dark1;
		}
	}
</style>
