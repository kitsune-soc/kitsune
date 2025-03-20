<script lang="ts">
	let { characterLimit }: { characterLimit: number } = $props();

	let content = $state('');
	let remainingCharacters = $derived(characterLimit - content.length);
	let isOverLimit = $derived(remainingCharacters < 0);
</script>

<div class="card bg-base-300 shadow-xl">
	<div class="card-body gap-5">
		<textarea
			bind:value={content}
			class="textarea w-full bg-transparent text-inherit"
			placeholder="Scream into the void..."
		></textarea>

		<div class="flex items-center justify-between">
			<div class:text-error={isOverLimit}>{remainingCharacters}</div>
			<button class="btn btn-primary" disabled={isOverLimit}>Post</button>
		</div>
	</div>
</div>
