<script lang="ts">
	import type { Visibility$options } from '$houdini';
	import { GQL_CreatePost } from '$houdini';

	import { Select } from 'bits-ui';
	import { values } from 'lodash';
	import { toast } from 'svelte-sonner';

	import { PostVisibility } from './post';

	let { characterLimit, onnewpost }: { characterLimit: number; onnewpost?: () => void } = $props();

	let content = $state('');

	let remainingCharacters = $derived(characterLimit - content.length);
	let isOverLimit = $derived(remainingCharacters < 0);
	let postDisabled = $derived(content.length === 0 || isOverLimit);

	let errors: string[] | undefined = $state();

	const visibilityOptions: Record<Visibility$options, { name: string }> = {
		PUBLIC: { name: 'Public' },
		UNLISTED: { name: 'Unlisted' },
		FOLLOWER_ONLY: { name: 'Followers only' },
		MENTION_ONLY: { name: 'Mention only' }
	};
	const items = Object.entries(visibilityOptions).map(([value, { name: label }]) => ({
		value,
		label
	}));

	let visibility: Visibility$options = $state('PUBLIC');

	async function submitPost(): Promise<void> {
		const result = await GQL_CreatePost.mutate({
			content,
			visibility
		});

		if (result.errors) {
			errors = result.errors.map((error) => error.message);
		} else {
			errors = undefined;

			content = '';
			if (onnewpost) onnewpost();

			toast.success('Post created!');
		}
	}
</script>

<div class="card bg-base-300 shadow-xl">
	<div class="card-body gap-5">
		{#if errors}
			<div class="alert alert-error">
				<ul class="m-0 list-none p-0">
					{#each errors as error, idx (idx)}
						<li>{error}</li>
					{/each}
				</ul>
			</div>
		{/if}

		<textarea
			bind:value={content}
			class="textarea w-full bg-transparent text-inherit"
			placeholder="Scream into the void..."
		></textarea>

		<div class="flex items-center justify-between">
			<div>
				<span class:text-error={isOverLimit}>{remainingCharacters}</span>
			</div>

			<div>
				<Select.Root type="single" bind:value={visibility} {items}>
					<Select.Trigger class="btn btn-neutral m-1">
						<PostVisibility halfVisible={false} {visibility} />
						{visibilityOptions[visibility].name}
					</Select.Trigger>

					<Select.Portal>
						<Select.Content class="bg-base-100 rounded-box shadow-xl">
							<ul class="menu m-0 p-2">
								{#each items as item, i (i + item.value)}
									<Select.Item value={item.value} label={item.label}>
										<li class="m-0">
											<button>
												<PostVisibility
													halfVisible={false}
													visibility={item.value as Visibility$options}
												/>
												{item.label}
											</button>
										</li>
									</Select.Item>
								{/each}
							</ul>
						</Select.Content>
					</Select.Portal>
				</Select.Root>

				<button class="btn btn-primary" onclick={() => submitPost()} disabled={postDisabled}>
					Post
				</button>
			</div>
		</div>
	</div>
</div>
