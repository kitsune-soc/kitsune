<script lang="ts" module>
	import ProfileCard from '$lib/components/ProfileCard.svelte';
	import { defineMeta } from '@storybook/addon-svelte-csf';

	import profilePic from './assets/profile_pic.png';

	const { Story } = defineMeta({
		title: 'Profile Card',
		component: ProfileCard,
		tags: ['autodocs']
	});
</script>

<Story
	name="Example"
	args={{
		avatarUrl: profilePic,
		name: 'Example',
		username: '@example@kitsune.example',
		description: 'Example Description',
		createdAt: new Date()
	}}
/>
