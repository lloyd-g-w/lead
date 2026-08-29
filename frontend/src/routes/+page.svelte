<script lang="ts">
	import Grid from '$lib/components/grid/grid.svelte';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import { fade } from 'svelte/transition';

	import { useSidebar } from '$lib/components/ui/sidebar/index.js';
	const sidebar = useSidebar();

	let socket = new WebSocket('ws://localhost:7050');

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
			<Grid class="h-full min-w-0" {socket} bind:wrapperEl={gridWrapper} />
		</div>
	</div>
</div>
