<script lang="ts">
	import '../app.css';
	import { ModeWatcher } from 'mode-watcher';
	import { Toaster } from '$lib/components/ui/sonner/index.js';
	import favicon from '$lib/assets/favicon.svg';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import LeadSidebar from '$lib/components/ui/lead-sidebar/lead-sidebar.svelte';

	let { children } = $props();
</script>

<ModeWatcher defaultMode="light" />
<Toaster richColors position="bottom-right" />
<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<Sidebar.Provider>
	<LeadSidebar />

	<main>
		{@render children?.()}
	</main>
</Sidebar.Provider>
