<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();

	let zoomed = $state(false);
</script>

{#if zoomed}
	<dialog open class="modal">
		<div class="modal-box p-0">
			{@render children()}
		</div>

		<form method="dialog" class="modal-backdrop" onsubmit={() => (zoomed = false)}>
			<button class="!cursor-zoom-out">close</button>
		</form>
	</dialog>
{:else}
	<button class="contents cursor-zoom-in" onclick={() => (zoomed = true)}>
		{@render children()}
	</button>
{/if}
