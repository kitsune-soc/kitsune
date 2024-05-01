<script lang="ts">
	import Button from '$lib/components/Button.svelte';
	import { onMount } from 'svelte';

	onMount(() => {
		// TODO: Authenticated check and redirect to home timeline
	});
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
				<form class="register-form">
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
						<Button class="register-button" onclick={() => console.log('register')}>
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
