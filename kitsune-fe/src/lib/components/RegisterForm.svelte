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
	<div>
		<label for="username"> Username </label>
		<Input type="text" name="username" placeholder="hangaku" required disabled={processing} />
	</div>

	<div>
		<label for="email"> Email address </label>
		<Input
			type="email"
			name="email"
			placeholder="hangaku@kabuki.dd"
			required
			disabled={processing}
		/>
	</div>

	<div>
		<label for="password"> Password </label>
		<Input type="password" name="password" required disabled={processing} />
	</div>

	<div>
		<label for="confirm-password"> Confirm Password </label>
		<Input type="password" name="confirm-password" required disabled={processing} />
	</div>

	<Button class="w-full" loading={processing}>Register</Button>
</form>
