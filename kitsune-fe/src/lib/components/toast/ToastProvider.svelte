<script lang="ts" module>
	import { Toaster } from 'melt/builders';

	type ToastData = {
		severity: 'info' | 'error' | 'success';
		message: string;
	};

	const toaster = new Toaster<ToastData>();
	const pushToast = toaster.addToast;

	export { pushToast };
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';
	import { fade } from 'svelte/transition';

	let { children }: { children: Snippet } = $props();
</script>

<div {...toaster.root} class="toast toast-center">
	{#each toaster.toasts as toast (toast.id)}
		<div
			{...toast.content}
			transition:fade
			class="alert"
			class:alert-error={toast.data.severity === 'error'}
			class:alert-info={toast.data.severity === 'info'}
			class:alert-success={toast.data.severity === 'success'}
		>
			<span {...toast.description}>
				{toast.data.message}
			</span>
		</div>
	{/each}
</div>

{@render children()}
