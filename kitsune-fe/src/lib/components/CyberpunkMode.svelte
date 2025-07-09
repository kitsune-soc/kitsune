<script lang="ts">
	import { uiSettings } from '$lib/ui-settings';
</script>

<label class="flex cursor-pointer gap-2">
	<input
		type="checkbox"
		value="cyberpunk"
		bind:checked={$uiSettings.cyberpunkMode}
		class="toggle theme-controller"
	/>
	<span class="label-text">Cyberpunk mode</span>
</label>
