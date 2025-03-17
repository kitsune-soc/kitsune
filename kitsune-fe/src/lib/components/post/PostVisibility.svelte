<script lang="ts">
	import type { Visibility$options } from '$houdini';

	import IconGlobe from '~icons/mdi/globe';
	import IconLockOpen from '~icons/mdi/lock-open';
	import IconLock from '~icons/mdi/lock';
	import IconEmailOutline from '~icons/mdi/email-outline';

	let { visibility }: { visibility: Visibility$options } = $props();

	let tooltip = $derived.by(() => {
		switch (visibility) {
			case 'PUBLIC':
				return 'Public';
			case 'UNLISTED':
				return 'Unlisted';
			case 'FOLLOWER_ONLY':
				return 'Follower only';
			case 'MENTION_ONLY':
				return 'Mentioned only';
		}
	});

	let icon = $derived.by(() => {
		switch (visibility) {
			case 'PUBLIC':
				return IconGlobe;
			case 'UNLISTED':
				return IconLockOpen;
			case 'FOLLOWER_ONLY':
				return IconLock;
			case 'MENTION_ONLY':
				return IconEmailOutline;
		}
	});
</script>

<span class="opacity-50" title={tooltip}>
	<icon class="opacity-50"></icon>
</span>
