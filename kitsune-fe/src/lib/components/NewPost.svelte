<script lang="ts">
	import type { Visibility$options } from '$houdini';
	import { GQL_CreatePost } from '$houdini';

	import { Select } from 'melt/builders';

	import { PostVisibility } from './post';
	import { pushToast } from './toast';

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

	const visibilitySelect = new Select<Visibility$options>({
		sameWidth: false,
		value: 'PUBLIC'
	});

	let visibility = $derived(visibilitySelect.value!);

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

			pushToast({
				severity: 'success',
				message: 'Post created!'
			});
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
				<button {...visibilitySelect.trigger} class="btn btn-neutral m-1">
					<PostVisibility halfVisible={false} {visibility} />
					{visibilityOptions[visibility].name}
				</button>

				<div class="bg-base-100 rounded-box shadow-xl" {...visibilitySelect.content}>
					<ul class="menu m-0 p-2">
						{#each Object.entries(visibilityOptions) as [key, value] (key)}
							<li>
								<button {...visibilitySelect.getOption(key as Visibility$options)}>
									<PostVisibility halfVisible={false} visibility={key as Visibility$options} />
									{value.name}
								</button>
							</li>
						{/each}
					</ul>
				</div>

				<button class="btn btn-primary" onclick={() => submitPost()} disabled={postDisabled}>
					Post
				</button>
			</div>
		</div>
	</div>
</div>
