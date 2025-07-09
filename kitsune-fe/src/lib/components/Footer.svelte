<script lang="ts">
	import Hangaku from '$assets/Hangaku.svelte';

	let { backendVersion, frontendVersion }: { backendVersion: string; frontendVersion: string } =
		$props();
</script>

<footer class="text-sm max-lg:text-center lg:fixed lg:bottom-3 lg:left-3">
	<p>
		Backend version: {backendVersion}
		<br />Frontend version: {frontendVersion}
	</p>

	<span>
		Powered by
		<a target="_blank" href="https://github.com/kitsune-soc/kitsune">Kitsune</a>
	</span>
</footer>

<div class="fixed right-3 bottom-3 hidden w-64 -scale-x-100 2xl:flex">
	<Hangaku />
</div>
