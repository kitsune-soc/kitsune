<script lang="ts">
	import { Button, Input } from './input';

	let {
		onregister,
		processing
	}: {
		onregister: (event: SubmitEvent & { currentTarget: EventTarget & HTMLFormElement }) => void;
		processing: boolean;
	} = $props();
</script>

<form
	class="grid grid-cols-1 gap-6"
	onsubmit={(e) => {
		e.preventDefault();
		onregister(e);
	}}
>
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
