<script lang="ts" module>
	import RegisterFormComponent from '$lib/components/RegisterForm.svelte';
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import { fn } from '@storybook/test';

	const { Story } = defineMeta({
		title: 'RegisterForm',
		component: RegisterFormComponent,
		tags: ['autodocs'],
		args: {
			onregister: fn()
		}
	});
</script>

<Story name="Default" />
