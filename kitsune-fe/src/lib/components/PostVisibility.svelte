<script lang="ts">
	import type { Visibility$options } from '$houdini';
	import Icon from '@iconify/svelte';

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

	let iconName = $derived.by(() => {
		switch (visibility) {
			case 'PUBLIC':
				return 'material-symbols:globe';
			case 'UNLISTED':
				return 'material-symbols:lock-open-right';
			case 'FOLLOWER_ONLY':
				return 'material-symbols:lock';
			case 'MENTION_ONLY':
				return 'material-symbols:mail';
		}
	});
</script>

<span title={tooltip}>
	<Icon class="h-auto w-5 opacity-50" icon={iconName} />
</span>
