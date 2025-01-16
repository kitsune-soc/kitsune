<script lang="ts">
	import Button from './Button.svelte';
	import Input from './Input.svelte';

	let {
		onregister,
		processing
	}: {
		onregister: (event: SubmitEvent & { currentTarget: EventTarget & HTMLFormElement }) => void;
		processing: boolean;
	} = $props();
</script>

<form class="grid grid-cols-1 gap-6" onsubmit={onregister}>
	<label class="block" for="username">
		Username
		<Input type="text" name="username" placeholder="hangaku" disabled={processing} />
	</label>

	<label for="email">
		Email address
		<Input type="email" name="email" placeholder="hangaku@kabuki.dd" disabled={processing} />
	</label>

	<label for="password">
		Password
		<Input type="password" name="password" disabled={processing} />
	</label>

	<label for="confirm-password">
		Confirm Password
		<Input type="password" name="confirm-password" disabled={processing} />
	</label>

	<p>
		<Button class="w-full" loading={processing}>Register</Button>
	</p>
</form>
