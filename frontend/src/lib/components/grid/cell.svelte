<script lang="ts">
	import { Input } from '$lib/components/ui/input/index.js';
	import clsx from 'clsx';

	let {
		width = '80px',
		height = '30px',
		raw_val = $bindable(''),
		val = undefined,
		onmousedown = () => {},
		startediting = () => {},
		stopediting = () => {},
		active = false,
		editing = false
	}: {
		width?: string;
		height?: string;
		raw_val?: string;
		val?: number | string | undefined;
		onmousedown?: (e: MouseEvent) => void;
		startediting?: () => void;
		stopediting?: () => void;
		active?: boolean;
		editing?: boolean;
	} = $props();

	// focus the first focusable descendant (the inner <input>)
	function autofocusWithin(node: HTMLElement) {
		queueMicrotask(() => {
			const el = node.querySelector('input') as HTMLInputElement | null;
			if (el !== null) {
				el.value = raw_val;
				el.focus();
			}
		});
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' || e.key === 'NumpadEnter' || e.key == 'Escape') {
			e.preventDefault(); // avoid form submit/line break
			const el = (e.currentTarget as HTMLElement).querySelector('input') as HTMLInputElement | null;
			el?.blur(); // triggers on:blur below
		}
	}
</script>

{#if editing}
	<div use:autofocusWithin onkeydown={handleKeydown}>
		<Input
			style="width: {width}; height: {height}"
			class="relative rounded-none p-1
             !transition-none delay-0 duration-0
             focus:z-50"
			onblur={(e) => {
				raw_val = (e.target as HTMLInputElement).value;
				stopediting();
			}}
		/>
	</div>
{:else}
	<div
		ondblclick={startediting}
		{onmousedown}
		style:width
		style:height
		class={clsx('placeholder bg-background p-1 dark:bg-input/30', { active, 'z-50': active })}
	>
		{#if raw_val !== '' || val !== ''}
			<span class="pointer-events-none select-none">
				{#if val !== undefined}
					{val}
				{:else}
					{raw_val}
				{/if}
			</span>
		{/if}
	</div>
{/if}

<style>
	.placeholder {
		border: 1px solid var(--input);
		overflow: hidden;
	}

	.active {
		border: 1px solid var(--color-primary);
		outline: 1px solid var(--color-primary);
	}
</style>
