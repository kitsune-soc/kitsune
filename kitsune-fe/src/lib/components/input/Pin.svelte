<script lang="ts">
	import { PinInput } from 'melt/builders';

	let {
		digits,
		disabled,
		onComplete,
		type,
		value = $bindable()
	}: {
		digits?: number;
		disabled?: boolean;
		onComplete?: (text: string) => void;
		type?: 'alphanumeric' | 'numeric';
		value?: string;
	} = $props();

	// builder is used on purpose since apparently the component version can't fire an `onComplete` event!
	const pinInput = $derived(
		new PinInput({
			disabled,
			maxLength: digits,
			onComplete,
			type,

			value,
			onValueChange: (newValue) => (value = newValue)
		})
	);
</script>

<div class="flex gap-2" {...pinInput.root}>
	{#each pinInput.inputs as input, idx (idx)}
		<input {...input} class="input w-9" />
	{/each}
</div>
