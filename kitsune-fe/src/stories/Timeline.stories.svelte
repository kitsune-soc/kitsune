<script lang="ts" module>
	import Timeline from '$lib/components/Timeline.svelte';
	import type { Post } from '$lib/types/Post';
	import { faker } from '@faker-js/faker';
	import { defineMeta } from '@storybook/addon-svelte-csf';

	import exampleAvatar from './assets/profile_pic.png';

	function generateRandomPost(): Post {
		return {
			id: faker.string.uuid(),
			user: {
				id: faker.string.uuid(),
				name: faker.person.fullName(),
				username: `@${faker.internet.username()}@${faker.internet.domainName()}`,
				avatarUrl: exampleAvatar
			},
			content: faker.hacker.phrase(),
			createdAt: new Date(),
			replyCount: faker.number.int({ max: 20 }),
			repostCount: faker.number.int({ max: 100 }),
			likeCount: faker.number.int({ max: 400 })
		};
	}

	let posts = new Array(10_000).fill(true).map(() => generateRandomPost());

	const { Story } = defineMeta({
		title: 'Timeline',
		component: Timeline,
		tags: ['autodocs']
	});
</script>

<Story
	name="Example"
	args={{
		posts
	}}
/>
