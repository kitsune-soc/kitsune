<script lang="ts">
	import { goto } from '$app/navigation';
	import Logo from '$assets/Logo.svelte';
	import { RegisterUserStore } from '$houdini';
	import Button from '$lib/components/Button.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import RegisterForm from '$lib/components/RegisterForm.svelte';
	import { loadOAuthApp } from '$lib/oauth/client.svelte';
	import { loadOAuthToken } from '$lib/oauth/token.svelte';
	import { registerSchema } from '$lib/schemas/register';

	import type { PageData } from './$houdini';

	const { data }: { data: PageData } = $props();

	const statsStore = $derived(data.stats);
	const stats = $derived({
		postCount: $statsStore.data?.instance.localPostCount ?? 0,
		registeredUsers: $statsStore.data?.instance.userCount ?? 0,
		registrationsOpen: $statsStore.data?.instance.registrationsOpen ?? true
	});

	const register = new RegisterUserStore();

	let registerButtonDisabled = $state(false);
	let registerErrors: string[] = $state([]);
	let registerErrorDialogOpen = $state(false);

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
			registerErrorDialogOpen = true;
			registerButtonDisabled = false;

			return;
		}

		try {
			const result = await register.mutate(validatedData.data);
			if (result.errors) {
				registerErrors = result.errors.map((error) => error.message);
				registerErrorDialogOpen = true;
			} else {
				event.currentTarget.reset();
				initiateLogin();
			}
		} catch (reason: unknown) {
			if (reason instanceof Error) {
				registerErrors = [reason.message];
			}

			registerErrorDialogOpen = true;
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
		oauthUrl.searchParams.set('redirect_uri', encodeURIComponent(oauthApp.redirectUri));
		oauthUrl.searchParams.set('scope', encodeURIComponent(['read', 'write', 'follow'].join(' ')));

		window.location.assign(oauthUrl);
	}

	loadOAuthToken().then((token) => {
		if (!token) {
			return;
		}

		goto('/timeline/home');
	});
</script>

<Dialog isOpen={registerErrorDialogOpen}>
	<h2>Registration failed!</h2>

	{#if registerErrors.length > 0}
		<ol>
			{#each registerErrors as error}
				<li>{error}</li>
			{/each}
		</ol>
	{/if}

	<button
		class="border-grey rounded-md border-2 px-2 py-1"
		onclick={() => (registerErrorDialogOpen = false)}
	>
		Close
	</button>
</Dialog>

<div
	class="flex min-h-screen flex-col max-lg:mt-5 lg:flex-row lg:place-content-evenly lg:items-center"
>
	<div class="flex basis-1/4 flex-col max-lg:place-items-center max-lg:text-center">
		<Logo class=" max-w-3/4" />

		<h1>Federated microblogging</h1>

		Statistics:

		<ul class="list-none p-0">
			<li>
				<strong>{stats.registeredUsers}</strong> registered users
			</li>
			<li>
				<strong>{stats.postCount}</strong> posts
			</li>
		</ul>
	</div>

	<div class="z-10 basis-1/4 max-lg:m-5">
		{#if stats.registrationsOpen}
			<RegisterForm onregister={doRegister} processing={registerButtonDisabled} />
		{/if}

		<Button class="w-full" buttonType="secondary" onclick={initiateLogin} loading={loginInProcess}>
			Already have an account? Sign in
		</Button>
	</div>
</div>
