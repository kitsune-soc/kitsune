<script lang="ts">
	let { backendVersion, frontendVersion }: { backendVersion: string; frontendVersion: string } =
		$props();
</script>

<footer class="w-full text-sm max-lg:mb-5 max-lg:text-center lg:fixed lg:bottom-3 lg:left-3">
	<p>
		Backend version: {backendVersion}
		<br />Frontend version: {frontendVersion}
	</p>

	<span>
		Powered by
		<a target="_blank" href="https://github.com/kitsune-soc/kitsune">Kitsune</a>
	</span>
</footer>
