<script lang="ts">
	import { Input } from '$lib/components/ui/input/index.js';
	import clsx from 'clsx';

	let {
		cla = '',
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
		cla?: string;
		width?: string;
		height?: string;
		raw_val?: string;
		val?: LiteralValue | undefined;
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
		if (e.key === 'Enter' || e.key === 'NumpadEnter') {
			e.preventDefault(); // avoid form submit/line break
			const el = (e.currentTarget as HTMLElement).querySelector('input') as HTMLInputElement | null;
			el?.blur(); // triggers on:blur below
		} else if (e.key == 'Escape') {
			e.preventDefault();
			stopediting();
		}
	}
</script>

{#if editing}
	<div use:autofocusWithin onkeydown={handleKeydown}>
		<Input
			style="width: {width}; height: {height}"
			class="relative rounded-none p-1 !transition-none delay-0 duration-0
			focus:z-20 focus:shadow-[0_0_0_1px_var(--color-primary)] focus:outline-none"
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
		class={clsx('placeholder bg-background p-1', { active }, cla)}
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
		white-space: nowrap;
		overflow: hidden;
		text-overflow: clip;
	}

	.active {
		z-index: 20;
		border: 1px solid var(--color-primary);
		outline: 1px solid var(--color-primary);
	}
</style>
