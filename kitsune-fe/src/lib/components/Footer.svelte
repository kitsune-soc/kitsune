<script lang="ts">
	import Hangaku from '$assets/Hangaku.svelte';

	let { backendVersion, frontendVersion }: { backendVersion: string; frontendVersion: string } =
		$props();
</script>

<footer
	class="-z-50 w-full place-content-between place-items-end text-sm max-lg:mb-5 max-lg:text-center lg:fixed lg:bottom-3 lg:left-3 lg:flex"
>
	<div>
		<p>
			Backend version: {backendVersion}
			<br />Frontend version: {frontendVersion}
		</p>

		<span>
			Powered by
			<a target="_blank" href="https://github.com/kitsune-soc/kitsune">Kitsune</a>
		</span>
	</div>

	<Hangaku class="hidden w-64 -scale-x-100 xl:inline" />
</footer>
