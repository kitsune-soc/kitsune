<script lang="ts">
	import { goto } from "$app/navigation";
	import { clearTokenStorage, tokenStore } from "$lib/oauth/token";
	import Icon from "@iconify/svelte";

    let { drawerItemName }: { drawerItemName: string; } = $props();

    const isLoggedIn = $derived($tokenStore !== undefined);

    function logout() {
        clearTokenStorage();
        goto("/");
    }
</script>

<nav class="navbar bg-base-300 w-full not-prose fixed">
    <div class="flex-none">
        <label for={drawerItemName} aria-label="open sidebar" class="btn btn-square btn-ghost">
            <Icon class="h-6 w-auto" icon="mdi:menu" />
        </label>
    </div>
    <div class="mx-2 flex-1 px-2 text-3xl font-bold">
        Kitsune
    </div>
    <div class="hidden flex-none lg:block">
        <ul class="menu menu-horizontal">
            {#if isLoggedIn}
                <li>
                    <a href="#" onclick={logout}><Icon class="h-5 w-auto" icon="mdi:logout" /> Logout</a>
                </li>
            {/if}
        </ul>
    </div>
</nav>
