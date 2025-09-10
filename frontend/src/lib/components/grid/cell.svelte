<script lang="ts">
	import { Input } from '$lib/components/ui/input/index.js';
	import clsx from 'clsx';
	import { getErrDesc, getErrTitle, getEvalLiteral, isErr } from './utils';
	import * as HoverCard from '$lib/components/ui/hover-card/index.js';
	import type { CellT } from './messages';

	let {
		cla = '',
		width = '80px',
		height = '30px',
		cell = $bindable(undefined),
		onmousedown = () => {},
		startediting = () => {},
		stopediting = () => {},
		active = false,
		editing = false,
		externalediting = false
	}: {
		cla?: string;
		width?: string;
		height?: string;
		cell?: CellT;
		onmousedown?: (e: MouseEvent) => void;
		startediting?: () => void;
		stopediting?: () => void;
		active?: boolean;
		editing?: boolean;
		externalediting?: boolean;
	} = $props();

	// focus the first focusable descendant (the inner <input>)
	function autofocusWithin(node: HTMLElement) {
		queueMicrotask(() => {
			const el = node.querySelector('input') as HTMLInputElement | null;
			if (el !== null) {
				el.value = cell?.temp_raw ?? '';
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

	function getPreview() {
		return !isErr(cell?.temp_eval) ? getEvalLiteral(cell?.temp_eval) : '';
	}

	let showPreview = $derived(getPreview() !== '');
</script>

{#if editing}
	<div class="relative inline-block">
		{#if showPreview}
			<h3
				class="bubble pointer-events-none absolute -top-[6px] -left-1 z-[500] -translate-y-full text-sm font-semibold tracking-tight text-foreground select-none"
				role="tooltip"
			>
				{getPreview()}
			</h3>
		{/if}

		<div use:autofocusWithin onkeydown={handleKeydown}>
			<Input
				style="width: {width}; height: {height}"
				class="relative rounded-none p-1 !transition-none delay-0 duration-0
        focus:z-20 focus:shadow-[0_0_0_1px_var(--color-primary)] focus:outline-none"
				bind:value={
					() => cell?.temp_raw ?? '',
					(v) => (cell = { eval: cell?.eval, raw: cell?.raw ?? '', temp_raw: v })
				}
				onblur={stopediting}
			/>
		</div>
	</div>
{:else if cell && isErr(cell.eval)}
	<HoverCard.Root openDelay={500} closeDelay={500}>
		<HoverCard.Trigger>
			{@render InnerCell()}
		</HoverCard.Trigger>
		<HoverCard.Content side="right">
			<h2 class="text-md font-semibold tracking-tight transition-colors">
				{getErrTitle(cell.eval)}
			</h2>
			{getErrDesc(cell.eval)}
		</HoverCard.Content>
	</HoverCard.Root>
{:else}
	{@render InnerCell()}
{/if}

{#snippet InnerCell()}
	<div
		ondblclick={startediting}
		{onmousedown}
		style:width
		style:height
		class={clsx('placeholder bg-background p-1', { active }, cla)}
	>
		{#if cell && (cell.raw !== '' || getEvalLiteral(cell.eval) !== '')}
			<span class={clsx('pointer-events-none select-none', { err: isErr(cell.eval) })}>
				{#if cell.eval && !externalediting}
					{getEvalLiteral(cell.eval)}
				{:else}
					{cell.raw}
				{/if}
			</span>
		{/if}
	</div>
{/snippet}

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

	.active:has(.err),
	.placeholder:has(.err) {
		position: relative; /* needed for absolute positioning */
		color: red;
	}

	.active:has(.err)::after,
	.placeholder:has(.err)::after {
		content: '';
		position: absolute;
		top: 0;
		right: 0;
		width: 0;
		height: 0;
		border-top: 12px solid red; /* size & color of the triangle */
		border-left: 12px solid transparent;
	}

	.bubble {
		z-index: 500;
		background: var(--color-popover);
		border: 1px solid var(--color-border, rgba(0, 0, 0, 0.12));
		border-radius: 10px;
		color: var(--color-popover-foreground);
		padding: 0.35rem 0.6rem;
		box-shadow: 0 2px 18px rgba(0, 0, 0, 0.08);
		max-width: min(15rem, 20vw);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		line-height: 1.2;
	}

	/* (optional) subtle appear animation */
	@media (prefers-reduced-motion: no-preference) {
		.bubble {
			transform-origin: bottom left;
			animation: bubble-in 120ms ease-out both;
		}
		@keyframes bubble-in {
			from {
				opacity: 0;
				transform: translateY(2px) scale(0.98);
			}
			to {
				opacity: 1;
				transform: translateY(0) scale(1);
			}
		}
	}
</style>
