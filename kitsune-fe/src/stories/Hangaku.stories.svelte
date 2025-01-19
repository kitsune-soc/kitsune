<script lang="ts" module>
	import HangakuComponent from '$assets/Hangaku.svelte';
	import { defineMeta } from '@storybook/addon-svelte-csf';

	const { Story } = defineMeta({
		title: 'Hangaku',
		component: HangakuComponent
	});
</script>

<Story name="Hangaku" />
