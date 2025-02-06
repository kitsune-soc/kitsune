<script lang="ts" module>
	import FooterComponent from '$lib/components/Footer.svelte';
	import { defineMeta } from '@storybook/addon-svelte-csf';

	const { Story } = defineMeta({
		title: 'Footer',
		component: FooterComponent
	});
</script>

<Story
	name="Footer"
	args={{
		backendVersion: '[Backend version]',
		frontendVersion: '[Frontend version]'
	}}
/>
