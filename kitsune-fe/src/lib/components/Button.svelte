<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		children,
		...rest
	}:
		| {
				children: Snippet;
		  }
		| any = $props();
</script>

<button {...rest}>
	{@render children()}
</button>

<style lang="scss">
	@use '../../styles/colours' as *;

	button {
		transition: 0.5s;
		cursor: pointer;

		border: 0;
		border-radius: 10px;

		background-color: $shade1dark;

		padding: 10px;
		width: 100px;
		height: 50px;
		font-size: 16px;

		&:hover {
			background-color: $shade2dark;
		}
	}
</style>
