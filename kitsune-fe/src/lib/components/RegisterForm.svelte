<script lang="ts">
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
	<label for="username">
		Username
	</label>
	<input class="input w-full validator" type="text" name="username" placeholder="hangaku" required  disabled={processing} />
</div>

<div>
	<label for="email">
		Email address
	</label>
	<input class="input w-full validator" type="email" name="email" placeholder="hangaku@kabuki.dd" required disabled={processing} />
</div>

<div>
	<label for="password">
		Password
	</label>
	<input class="input w-full validator" type="password" name="password" required  disabled={processing} />
</div>

<div>
	<label for="confirm-password">
		Confirm Password
	</label>
	<input class="input w-full validator" type="password" name="confirm-password" required  disabled={processing} />
</div>

	<p>
		<button class="w-full btn btn-primary" loading={processing}>Register</button>
	</p>
</form>
