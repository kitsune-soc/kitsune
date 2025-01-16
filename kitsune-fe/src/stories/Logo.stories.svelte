<script lang="ts" module>
	import LogoComponent from '$assets/Logo.svelte';
	import { defineMeta } from '@storybook/addon-svelte-csf';

	const { Story } = defineMeta({
		title: 'Logo',
		component: LogoComponent
	});
</script>

<Story name="Logo" />
