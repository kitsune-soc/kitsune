<script lang="ts">
	import { DateTime } from 'luxon';

	let { time: rawTime }: { time: Date | string } = $props();

	let parsedTime = $state(new Date(rawTime));

	function formatRelative(date: Date): string {
		return DateTime.fromJSDate(date).toRelative()!;
	}

	let time = $state(formatRelative(parsedTime));

	$effect(() => {
		const interval = setInterval(() => {
			time = formatRelative(parsedTime);
		}, 3_000);

		return () => clearInterval(interval);
	});
</script>

{time}
