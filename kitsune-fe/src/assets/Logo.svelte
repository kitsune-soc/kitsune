<script lang="ts">
	import type { HTMLImgAttributes } from 'svelte/elements';

	import logoRaw from './kitsune_full.svg';

	let props: Omit<HTMLImgAttributes, 'src' | 'srcset'> = $props();
</script>

<img {...props} src={logoRaw} />
