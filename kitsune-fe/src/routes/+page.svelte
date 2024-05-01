<script lang="ts">
	import Button from '$lib/components/Button.svelte';
	import { onMount } from 'svelte';

	onMount(() => {
		// TODO: Authenticated check and redirect to home timeline
	});
</script>

<div class="landing-page">
	<div class="left-section">
		<img class="logo" alt="Kitsune VTuber-style logo" src="/kitsune_vtuber.png" />
		<h1>Federated microblogging.</h1>

		Instance statistics:

		<ul>
			<li>Registered users: 0</li>
			<li>Published posts: 0</li>
			<li>Known instances: 0</li>
		</ul>
	</div>

	<div class="right-section">
		<div class="login-form">
			<Button class="login-button" onclick={() => alert('clicked')}>Login</Button>
		</div>

		<h2>-- OR --</h2>

		<form
			class="register-form"
			onsubmit={(e) => {
				e.preventDefault();
				alert('submitted');
			}}
		>
			<label>
				Username
				<input name="username" type="text" />
			</label>

			<label>
				Email
				<input name="email" type="email" />
			</label>

			<label>
				Password
				<input name="password" type="password" />
			</label>

			<label>
				Confirm password
				<input name="password-confirm" type="password" />
			</label>

			<p>
				<Button>Register</Button>
			</p>
		</form>
	</div>
</div>

<style lang="scss">
	@use '../styles/colours' as *;
	@use '../styles/mixins' as *;

	.landing-page {
		display: flex;
		flex-direction: row;
		align-content: center;

		padding: 5em 8em;

		width: 100%;
		height: 100%;

		@include only-on-mobile {
			flex-direction: column;
		}
	}

	.left-section {
		display: flex;
		flex-direction: column;
		justify-content: center;

		flex: 40%;

		@include only-on-mobile {
			align-items: center;
		}
	}

	.right-section {
		display: flex;
		flex-direction: column;

		flex: 10%;
		padding: 1em;

		justify-content: center;
		align-items: center;
	}

	.logo {
		width: 60%;
	}

	.login-form,
	.register-form {
		border: 1px solid $shade1dark;
		border-radius: 0.5em;
	}

	.login-form {
		padding: 2em 7em;
		background-color: $dark3;
	}

	.register-form {
		padding: 2em;
		background-color: $dark3;

		& label {
			display: flex;
			flex-direction: column;
			width: 30ch;

			padding: 0.3em 0;
		}
	}
</style>
