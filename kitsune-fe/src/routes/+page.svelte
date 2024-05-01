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
	<div class="top-section">
		<div class="top-section-content">
			<div class="top-section-left">
				<img class="logo" src="/kitsune_vtuber.png" />

				<h1>Federated microblogging</h1>

				Statistics:

				<ul>
					<li>1,000,000,000 registered users</li>
					<li>96,000,000,000 posts</li>
					<li>50,000,000 connected instances</li>
				</ul>
			</div>

			<div class="top-section-right">
				<form
					class="register-form"
					onsubmit={(e) => {
						registerButtonDisabled = true;
						handleRegister(e).finally(() => (registerButtonDisabled = false));
					}}
				>
					<label>
						Username
						<br /><input type="text" name="username" />
					</label>

					<label>
						Email
						<br /><input type="email" name="email" />
					</label>

					<label>
						Password
						<br /><input type="password" name="password" />
					</label>

					<label>
						Confirm Password
						<br /><input type="password" name="confirm-password" />
					</label>

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
		</div>
	</div>

	<div class="bottom-section"></div>
</div>

<style lang="scss">
	@use '../styles/breakpoints' as *;
	@use '../styles/colours' as *;
	@use '../styles/mixins' as *;
	@use 'sass:map';

	$top-percentage: 70%;

	.landing-page {
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.top-section {
		display: flex;
		flex-direction: row;
		justify-content: center;
		align-items: flex-end;

		flex: $top-percentage;

		&-content {
			display: flex;
			flex-direction: row;
			justify-content: center;

			width: 100%;
			height: 80%;

			max-width: map.get($breakpoints, lg);
		}

		&-right {
			background-color: $dark2;
			padding: 3em;
		}
	}

	.bottom-section {
		background-color: $dark2;
		flex: calc(100% - $top-percentage);
	}

	.logo {
		width: 65%;
	}

	.register-form {
		display: flex;
		flex-direction: column;

		& label {
			margin: 0.3em 0;

			& input {
				width: 40ch;
			}
		}

		& :global(.register-button) {
			width: 100%;
			margin-top: 2em;
		}
	}

	@include only-on-mobile {
		.bottom-section {
			display: none;
		}

		.top-section-content {
			flex-direction: column;
			align-items: center;
		}

		.top-section-left {
			display: flex;
			flex-direction: column;
			align-items: center;
		}
	}
</style>
