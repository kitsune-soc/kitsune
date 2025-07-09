<script lang="ts" module>
	import { Pin } from '$lib/components/input';
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import { fn } from '@storybook/test';

	const { Story } = defineMeta({
		title: 'PinInput',
		component: Pin,
		tags: ['autodocs'],
		args: {
			onComplete: fn()
		}
	});
</script>

<Story name="Default" />
<Story name="Disabled" args={{ disabled: true }} />
