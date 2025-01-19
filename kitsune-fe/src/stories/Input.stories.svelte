<script lang="ts" module>
	import Input from '$lib/components/Input.svelte';
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import { fn } from '@storybook/test';

	const { Story } = defineMeta({
		title: 'Input',
		component: Input,
		tags: ['autodocs'],
		args: {
			oninput: fn()
		}
	});
</script>

<Story name="Default" args={{ placeholder: 'Default' }} />
<Story name="Disabled" args={{ placeholder: 'Disabled', disabled: true }} />
