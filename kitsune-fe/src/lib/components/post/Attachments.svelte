<script lang="ts">
	import type { MediaAttachment } from '$lib/types/MediaAttachment';

	import Zoom from '../Zoom.svelte';

	let { attachments }: { attachments: MediaAttachment[] } = $props();

	const mediaTypes = ['audio', 'image', 'video'] as const;
	type MediaType = (typeof mediaTypes)[number];

	function mediaType(attachment: MediaAttachment): MediaType | undefined {
		for (const mediaType of mediaTypes) {
			if (attachment.contentType.startsWith(mediaType)) {
				console.log('its mediatype: ' + mediaType);
				return mediaType;
			}
		}
	}
</script>

<div class="m-3 grid grid-cols-2 gap-3 first:row-end-[span_2]">
	{#each attachments as attachment, idx (idx)}
		<div class="h-full w-full" title={attachment.description}>
			{#if mediaType(attachment) === 'audio'}
				<audio class="w-full" src={attachment.url} controls volume={0.5}></audio>
			{:else if mediaType(attachment) === 'image'}
				<Zoom>
					<img class="m-0" alt={attachment.description} src={attachment.url} />
				</Zoom>
			{:else if mediaType(attachment) === 'video'}
				<!-- svelte-ignore a11y_media_has_caption -->
				<video src={attachment.url} controls volume={0.5}> </video>
			{:else if mediaType(attachment) === undefined}
				<div class="card bg-error text-error-content p-3">
					Unsupported content type: "{attachment.contentType}"
				</div>
			{/if}
		</div>
	{/each}
</div>
