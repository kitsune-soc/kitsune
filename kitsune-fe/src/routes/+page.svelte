<script lang="ts">
	import { RegisterUserStore } from '$houdini';
	import Button from '$lib/components/Button.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import type { PageData } from './$houdini';
	import IconThreeDotsLoading from '~icons/eos-icons/three-dots-loading?raw&width=2em&height=2em';
	import { _registerSchema } from './+page';

	const { data }: { data: PageData } = $props();

	const statsStore = $derived(data.stats);
	const stats = $derived({
		postCount: $statsStore.data?.instance.localPostCount ?? 0,
		registeredUsers: $statsStore.data?.instance.userCount ?? 0
	});

	const register = new RegisterUserStore();

	let registerButtonDisabled = $state(false);
	let registerErrors: string[] = $state([]);
	let registerErrorDialogOpen = $state(false);

	async function doRegister(event: SubmitEvent & { currentTarget: EventTarget & HTMLFormElement }) {
		event.preventDefault();
		registerButtonDisabled = true;

		const formData = new FormData(event.currentTarget);
		const validatedData = await _registerSchema.safeParseAsync(
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

	function initiateLogin() {
		alert('logging in wwowowowowowo');
	}
</script>

<Dialog isOpen={registerErrorDialogOpen}>
	<h2>Registration failed!</h2>

	<ol>
		{#each registerErrors as error}
			<li>{error}</li>
		{/each}
	</ol>

	<button
		class="border-grey rounded border-2 px-2 py-1"
		onclick={() => (registerErrorDialogOpen = false)}
	>
		Close
	</button>
</Dialog>

<div
	class="flex min-h-screen w-screen flex-col max-lg:mt-5 lg:flex-row lg:place-content-evenly lg:items-center"
>
	<div class="flex flex-col max-lg:place-items-center max-lg:text-center">
		<img class="w-3/5" src="/kitsune_full.svg" alt="Kitsune logo" />

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

	<div class="basis-1/4 max-lg:m-5">
		<form class="grid grid-cols-1 gap-6" onsubmit={doRegister}>
			<label class="block" for="username">
				Username

				<input
					class="w-full border-0 border-b-2 border-gray-200 bg-transparent"
					type="text"
					name="username"
					placeholder="hangaku"
				/>
			</label>

			<label for="email">
				Email address

				<input
					class="w-full border-0 border-b-2 border-gray-200 bg-transparent"
					type="email"
					name="email"
					placeholder="hangaku@kabuki.dd"
				/>
			</label>

			<label for="password">
				Password

				<input
					class="w-full border-0 border-b-2 border-gray-200 bg-transparent"
					type="password"
					name="password"
				/>
			</label>

			<label for="confirm-password">
				Confirm Password

				<input
					class="w-full border-0 border-b-2 border-gray-200 bg-transparent"
					type="password"
					name="confirm-password"
				/>
			</label>

			<p>
				<Button class="w-full" disabled={registerButtonDisabled}>
					{#if registerButtonDisabled}
						<!-- Work around unplugin-icons bug: <https://github.com/unplugin/unplugin-icons/issues/242> -->
						<!-- eslint-disable-next-line svelte/no-at-html-tags -->
						{@html IconThreeDotsLoading}
					{:else}
						Register
					{/if}
				</Button>
			</p>
		</form>

		<Button class="w-full" buttonType="secondary" onclick={initiateLogin}>
			Already have an account? Sign in
		</Button>
	</div>
</div>
