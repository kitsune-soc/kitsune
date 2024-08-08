<script lang="ts">
	import { RegisterUserStore } from '$houdini';
	import Button from '$lib/components/Button.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import type { PageData } from './$houdini';
	import IconThreeDotsLoading from '~icons/eos-icons/three-dots-loading?raw&width=2em&height=2em';

	const { data }: { data: PageData } = $props();

	const statsStore = $derived(data.stats);
	const stats = $derived({
		postCount: $statsStore.data?.instance.localPostCount ?? 0,
		registeredUsers: $statsStore.data?.instance.userCount ?? 0
	});

	const register = new RegisterUserStore();

	let registerButtonDisabled = $state(false);
	let registerError = $state();
	let registerErrorDialogOpen = $state(false);

	function doRegister(event: SubmitEvent & { currentTarget: EventTarget & HTMLFormElement }) {
		event.preventDefault();
		registerButtonDisabled = true;

		const formData = new FormData(event.currentTarget);

		const username = formData.get('username')!.toString();
		const email = formData.get('email')!.toString();
		const password = formData.get('password')!.toString();
		const confirmPassword = formData.get('confirm-password')!.toString();

		if (password !== confirmPassword) {
			registerError = 'Passwords do not match';
			registerErrorDialogOpen = true;
			registerButtonDisabled = false;
			return;
		}

		register
			.mutate({ username, email, password })
			.then((result) => {
				if (result.errors) {
					registerError = result.errors.map((error) => error.message).join(', ');
					registerErrorDialogOpen = true;
				} else {
					event.currentTarget.reset();
					initiateLogin();
				}
			})
			.catch((reason) => {
				registerError = reason.message;
				registerErrorDialogOpen = true;
			})
			.finally(() => {
				registerButtonDisabled = false;
			});
	}

	function initiateLogin() {
		alert('logging in wwowowowowowo');
	}
</script>

<Dialog isOpen={registerErrorDialogOpen}>
	<h2>Registration failed!</h2>

	<p>
		{registerError}
	</p>

	<button onclick={() => (registerErrorDialogOpen = false)}>Close</button>
</Dialog>

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
			<form class="register-form" onsubmit={doRegister}>
				<label for="username">Username</label>
				<input placeholder="hangaku" type="text" name="username" />

				<label for="email">Email address</label>
				<input placeholder="hangaku@joinkitsune.org" type="email" name="email" />

				<label for="password">Password</label>
				<input type="password" name="password" />

				<label for="confirm-password">Confirm Password</label>
				<input type="password" name="confirm-password" />

				<p>
					<Button class="register-button" disabled={registerButtonDisabled}>
						{#if registerButtonDisabled}
							<!-- Work around unplugin-icons bug: <https://github.com/unplugin/unplugin-icons/issues/242> -->
							{@html IconThreeDotsLoading}
						{:else}
							Register
						{/if}
					</Button>
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

	ul {
		padding: 0;
		list-style-type: none;
	}

	.landing-page {
		display: flex;
		flex-direction: row;

		@include not-on-mobile {
			height: 100%;
		}

		.section-left {
			display: flex;
			justify-content: center;
			align-items: center;
			width: 100%;
		}

		.section-right {
			display: flex;
			flex-direction: column;
			justify-content: center;
			background-color: $dark2;
			width: 100%;

			&-content {
				border-radius: 0px 30px 30px 0px;
				background-color: $dark1;
				padding: 2em;
				max-width: 40ch;

				& :global(.sign-in-button) {
					width: 100%;
				}
			}
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
			margin-bottom: 0.75em;
			border: none;
			border-radius: 10px;
			background-color: $dark2;
			padding-left: 1em;
			height: 40px;
		}

		& :global(.register-button) {
			margin-top: 1.5em;
			width: 100%;
		}
	}

	@include only-on-mobile {
		.landing-page {
			flex-direction: column;

			margin: auto;
			margin-top: 3em;
			max-width: 80vw;

			.section-left {
				text-align: center;
			}

			.section-right-content {
				background-color: $dark2;
				max-width: 100%;
			}
		}

		.register-form input {
			background-color: $dark1;
		}
	}
</style>
