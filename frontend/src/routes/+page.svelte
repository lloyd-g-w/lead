<script lang="ts">
	import Grid from '$lib/components/grid/grid.svelte';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import { fade } from 'svelte/transition';
	import { onDestroy, onMount } from 'svelte';

	import { useSidebar } from '$lib/components/ui/sidebar/index.js';
	const sidebar = useSidebar();

	// Create the socket in the browser only (this route is prerendered) and
	// close it when the page is destroyed.
	let socket: WebSocket | null = $state(null);

	onMount(() => {
		socket = new WebSocket('ws://localhost:7050');
	});

	onDestroy(() => {
		socket?.close();
	});

	let gridWrapper: HTMLDivElement | undefined = $state();
</script>

{#if sidebar?.openMobile || sidebar?.open}
	<button
		class="fixed inset-0 z-[700] bg-black/40"
		aria-label="Close sidebar"
		onclick={() => (sidebar.isMobile ? sidebar.setOpenMobile(false) : sidebar.setOpen(false))}
		transition:fade={{ duration: 100 }}
	></button>
{/if}

<div class="absolute left-0 min-h-0 w-full">
	<div class="flex h-[93vh] flex-col">
		<div class="h-[60px] w-full p-3">
			<div class="flex items-center gap-5">
				<Sidebar.Trigger />
			</div>
		</div>

		<div class="grid-wrapper min-h-0 w-[99dvw] flex-1" bind:this={gridWrapper}>
			{#if socket}
				<Grid class="h-full min-w-0" {socket} bind:wrapperEl={gridWrapper} />
			{/if}
		</div>
	</div>
</div>
