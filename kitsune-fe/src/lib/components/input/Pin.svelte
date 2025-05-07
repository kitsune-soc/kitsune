<script lang="ts">
	import {
		PinInput,
		REGEXP_ONLY_CHARS,
		REGEXP_ONLY_DIGITS,
		REGEXP_ONLY_DIGITS_AND_CHARS,
		type PinInputRootSnippetProps
	} from 'bits-ui';

	type CellProps = PinInputRootSnippetProps['cells'][0];

	let {
		digits = 4,
		disabled,
		onComplete,
		type = 'numeric',
		value = $bindable('')
	}: {
		digits?: number;
		disabled?: boolean;
		onComplete?: (text: string) => void;
		type?: 'alphabetic' | 'alphanumeric' | 'numeric';
		value?: string;
	} = $props();

	let pattern = $derived.by(() => {
		switch (type) {
			case 'alphabetic':
				return REGEXP_ONLY_CHARS;
			case 'alphanumeric':
				return REGEXP_ONLY_DIGITS_AND_CHARS;
			case 'numeric':
				return REGEXP_ONLY_DIGITS;
		}
	});
</script>

<PinInput.Root
	bind:value
	class="group/pininput flex gap-2"
	maxlength={digits}
	onComplete={() => {
		if (onComplete) onComplete(value);
	}}
	{pattern}
	{disabled}
>
	{#snippet children({ cells })}
		{#each cells as cell}
			{@render Cell(cell)}
		{/each}
	{/snippet}
</PinInput.Root>

{#snippet Cell(cell: CellProps)}
	<PinInput.Cell {cell} class="input w-9">
		{#if cell.char}
			<div>
				{cell.char}
			</div>
		{/if}

		{#if cell.hasFakeCaret}
			<div class="animate-caret-blink absolute inset-0 flex items-center justify-center">
				<div class="bg-base-content h-3/5 w-px"></div>
			</div>
		{/if}
	</PinInput.Cell>
{/snippet}
