<script lang="ts" setup>
	//import GlitchedElement from './GlitchedElement.vue';
	import '@fortawesome/fontawesome-free/css/all.min.css';

	const { detail, icon, to }: { detail: string | undefined; icon: string; to: string } = $props();
</script>

<a class="nav-bar-link" href={to} draggable="false">
	<span class={icon}></span>
	{#if detail}
		<span class="detail">{detail}</span>
	{/if}
</a>

<style lang="scss">
	@use '../../styles/colours' as *;
	@use '../../styles/mixins' as *;

	.nav-bar-link {
		display: inline-block;

		cursor: pointer;

		border-bottom: 4px solid;
		border-color: transparent;
		padding: 15px;
		color: $shade1dark;
		user-select: none;

		&:hover {
			color: white;
		}

		& .icon {
			margin-right: 10px;
		}

		@include only-on-mobile {
			& .detail {
				display: none;
			}

			& .icon {
				margin-right: 0px;
			}
		}

		&.router-link-active {
			border-image-source: linear-gradient(to left, $shade2light, $shade2dark);
			border-image-slice: 1;
			color: $shade2light;
		}
	}
</style>
