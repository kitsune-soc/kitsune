<script lang="ts" module>
	import Button from '$lib/components/Button.svelte';
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import { fn } from '@storybook/test';

	const { Story } = defineMeta({
		title: 'Button',
		component: Button,
		tags: ['autodocs'],
		args: {
			onclick: fn()
		}
	});
</script>

<!-- eslint-disable-next-line @typescript-eslint/no-explicit-any -->
{#snippet template(args: any)}
	<Button {...args}>
		{args.value}
	</Button>
{/snippet}

<Story name="Default" args={{ value: 'Enabled' }} children={template} />

<Story name="Primary" args={{ value: 'Primary', buttonType: 'primary' }} children={template} />
<Story
	name="Secondary"
	args={{ value: 'Secondary', buttonType: 'secondary' }}
	children={template}
/>
<Story name="Danger" args={{ value: 'Danger', buttonType: 'danger' }} children={template} />

<Story name="Disabled" args={{ value: 'Disabled', disabled: true }} children={template} />
<Story name="Loading" args={{ value: 'Loading', loading: true }} children={template} />
