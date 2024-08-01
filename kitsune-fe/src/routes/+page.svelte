<script lang="ts">
	import { graphql } from '$houdini';
	import Button from '$lib/components/Button.svelte';
	import { onMount } from 'svelte';

	let registerButtonDisabled = $state(false);

	onMount(() => {
		// TODO: Authenticated check and redirect to home timeline
	});

	async function handleRegister(
		event: SubmitEvent & { currentTarget: EventTarget & HTMLFormElement }
	) {
		event.preventDefault();

		const data = new FormData(event.currentTarget);

		const username = data.get('username');
		const email = data.get('email');
		const password = data.get('password');
		const passwordConfirmation = data.get('confirm-password');

		if (!username) {
			alert('Missing username');
			return;
		} else if (!email) {
			alert('Missing email');
			return;
		} else if (!password) {
			alert('Missing password');
			return;
		} else if (!passwordConfirmation || passwordConfirmation !== password) {
			alert('Password mismatch');
			return;
		}

		const register = graphql(`
			mutation Register($username: String!, $email: String!, $password: String!) {
				registerUser(username: $username, email: $email, password: $password) {
					username
					createdAt
				}
			}
		`);

		try {
			const response = await register.mutate({
				username: username as string,
				email: email as string,
				password: password as string
			});

			if (response.errors) {
				alert('Failed to register:\n' + response.errors.map((error) => error.message).concat('\n'));
			} else {
				alert('Registered!');
			}
		} catch (ex: unknown) {}
	}
</script>

<div class="landing-page">
	<div class="section-left">
		<div class="section-left-content">
			<img class="logo" src="/kitsune_full.svg" />

			<h1>Federated microblogging</h1>

			Statistics:

			<ul>
				<li>1,000,000,000 registered users</li>
				<li>96,000,000,000 posts</li>
				<li>50,000,000 connected instances</li>
			</ul>
		</div>
	</div>

	<div class="section-right">
		<div class="section-right-content">
			<form
				class="register-form"
				onsubmit={(e) => {
					registerButtonDisabled = true;
					handleRegister(e).finally(() => (registerButtonDisabled = false));
				}}
			>
				<input placeholder="Username" type="text" name="username" />
				<input placeholder="Email" type="email" name="email" />
				<input placeholder="Password" type="password" name="password" />
				<input placeholder="Confirm Password" type="password" name="confirm-password" />

				<p>
					<Button
						class="register-button"
						onclick={() => console.log('register')}
						disabled={registerButtonDisabled}
					>
						Register
					</Button>
				</p>
			</form>
		</div>
		<Button class="sign-up-button" href="/login">Already have an account? Sign in</Button>
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

			.sign-up-button {
				display: none;
			}

			&-content {
				border-radius: 0px 30px 30px 0px;
				background-color: $dark1;
				padding: 2em;
				max-width: 50ch;
				width: 100%;
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
		height: 100%;

		& input {
			width: 100%;
			border: none;
			height: 50px;
			border-radius: 10px;
			background-color: $dark2;
			margin-bottom: 0.75em;
			margin-top: 0.75em;
			padding-left: 1em;
		}

		& :global(.register-button) {
			width: 100%;
			margin-top: 2em;
		}
	}

	h1,
	ul {
		color: $text1;
	}

	@include only-on-mobile {
		.bottom-section {
			display: none;
		}

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
