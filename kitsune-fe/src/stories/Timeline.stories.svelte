<script lang="ts" module>
	import Timeline from '$lib/components/Timeline.svelte';
	import type { Post } from '$lib/types/Post';
	import { faker } from '@faker-js/faker';
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import { fn } from '@storybook/test';

	import exampleAvatar from './assets/profile_pic.png';
	import { Visibility } from '$houdini';

	function generateRandomPost(): Post {
		const id = faker.string.uuid();
		const visibilities = Object.values(Visibility);

		return {
			id,
			user: {
				id: faker.string.uuid(),
				name: faker.person.fullName(),
				username: `${faker.internet.username()}@${faker.internet.domainName()}`,
				avatarUrl: exampleAvatar
			},
			content: faker.hacker.phrase(),
			createdAt: new Date(),
			replyCount: faker.number.int({ max: 20 }),
			repostCount: faker.number.int({ max: 100 }),
			likeCount: faker.number.int({ max: 400 }),
			url: `/posts/${id}`,
			visibility: visibilities[Math.floor(Math.random() * visibilities.length)],
		};
	}

	let posts = new Array(5_000).fill(true).map(() => generateRandomPost());

	const { Story } = defineMeta({
		title: 'Timeline',
		component: Timeline,
		tags: ['autodocs'],
		args: {
			onendreached: fn()
		}
	});
</script>

<Story
	name="Example"
	args={{
		posts
	}}
/>
