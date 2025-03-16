<script lang="ts">
	import { goto } from '$app/navigation';
	import Logo from '$assets/Logo.svelte';
	import { RegisterUserStore } from '$houdini';
	import RegisterForm from '$lib/components/RegisterForm.svelte';
	import { Button } from '$lib/components/input';
	import { loadOAuthApp } from '$lib/oauth/client';
	import { tokenStore } from '$lib/oauth/token';
	import { registerSchema } from '$lib/schemas/register';
	import Icon from '@iconify/svelte';

	import type { PageData } from './$houdini';

	const { data }: { data: PageData } = $props();

	const statsStore = $derived(data.stats);
	const stats = $derived({
		characterLimit: $statsStore.data?.instance.characterLimit ?? 0,
		description: $statsStore.data?.instance.description ?? '',
		postCount: $statsStore.data?.instance.localPostCount ?? 0,
		registeredUsers: $statsStore.data?.instance.userCount ?? 0,
		registrationsOpen: $statsStore.data?.instance.registrationsOpen ?? true
	});

	const register = new RegisterUserStore();

	let registerButtonDisabled = $state(false);
	let registerErrors: string[] = $state([]);

	async function doRegister(event: SubmitEvent & { currentTarget: EventTarget & HTMLFormElement }) {
		registerButtonDisabled = true;

		const formData = new FormData(event.currentTarget);
		const validatedData = await registerSchema.safeParseAsync(
			Object.fromEntries(formData.entries())
		);

		if (!validatedData.success) {
			const formattedErrors = validatedData.error.format(
				(issue) => `${issue.path.join(', ')}: ${issue.message}`
			);

			registerErrors = Object.values(formattedErrors).flatMap((error) =>
				'_errors' in error ? error._errors : error
			);
			registerButtonDisabled = false;

			return;
		}

		try {
			const result = await register.mutate(validatedData.data);
			if (result.errors) {
				registerErrors = result.errors.map((error) => error.message);
			} else {
				initiateLogin();
			}
		} catch (reason: unknown) {
			if (reason instanceof Error) {
				registerErrors = [reason.message];
			}
		} finally {
			registerButtonDisabled = false;
		}
	}

	let loginInProcess = $state(false);

	async function initiateLogin() {
		loginInProcess = true;

		const oauthApp = await loadOAuthApp();
		const oauthUrl = new URL(`${window.location.origin}/oauth/authorize`);

		oauthUrl.searchParams.set('response_type', 'code');
		oauthUrl.searchParams.set('client_id', oauthApp.id);
		oauthUrl.searchParams.set('redirect_uri', oauthApp.redirectUri);
		oauthUrl.searchParams.set('scope', 'read write follow');

		window.location.assign(oauthUrl);
	}

	tokenStore.subscribe((newToken) => {
		if (!newToken) return;
		goto('/timeline/home');
	});
</script>

<div class="hero min-h-screen">
	<div class="hero-content w-full flex-col justify-between lg:flex-row">
		<div class="text-center lg:text-left">
			<Logo class="max-w-3/4" />

			<h1>Federated microblogging.</h1>

			<p>
				<!-- eslint-disable-next-line svelte/no-at-html-tags -->
				{@html stats.description}
			</p>
		</div>

		<div class="join join-vertical max-w-md gap-3">
			<div class="bg-base-100 stats shadow">
				<div class="stat place-items-center">
					<div class="stat-title">Registered Users</div>
					<div class="stat-value">
						{stats.registeredUsers}
					</div>
				</div>

				<div class="stat place-items-center">
					<div class="stat-title">Authored posts</div>
					<div class="stat-value">
						{stats.postCount}
					</div>
				</div>

				<div class="stat place-items-center">
					<div class="stat-title">Character limit</div>
					<div class="stat-value">
						{stats.characterLimit}
					</div>
				</div>
			</div>

			<div class="card bg-base-100 p-10 shadow-2xl">
				{#if stats.registrationsOpen}
					{#if registerErrors.length !== 0}
						<div role="alert" class="alert alert-error mb-5">
							<Icon class="h-6 w-auto opacity-70" icon="mdi:error-outline" />
							<ol class="list-none p-0">
								{#each registerErrors as error, index (index)}
									<li>{error}</li>
								{/each}
							</ol>
						</div>
					{/if}

					<RegisterForm onregister={doRegister} processing={registerButtonDisabled} />
					<div class="divider">OR</div>
				{/if}

				<Button
					class="w-full"
					buttonType="neutral"
					onclick={initiateLogin}
					loading={loginInProcess}
				>
					Already have an account? Sign in
				</Button>
			</div>
		</div>
	</div>
</div>
