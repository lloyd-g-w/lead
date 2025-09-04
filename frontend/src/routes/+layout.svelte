<script lang="ts">
	import '../app.css';
	import { ModeWatcher } from 'mode-watcher';
	import { Toaster } from '$lib/components/ui/sonner/index.js';
	import favicon from '$lib/assets/favicon.svg';

	let { children } = $props();
</script>

<ModeWatcher defaultMode="light" />
<Toaster richColors position="bottom-right" />
<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{@render children?.()}
