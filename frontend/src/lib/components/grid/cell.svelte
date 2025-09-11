<script lang="ts">
	import { Input } from '$lib/components/ui/input/index.js';
	import clsx from 'clsx';
	import { getErrDesc, getErrTitle, getEvalLiteral, isErr } from './utils';
	import * as HoverCard from '$lib/components/ui/hover-card/index.js';
	import { Position, type Grid } from './grid.svelte.ts';

	let {
		cla = '',
		pos,
		onmousedown = () => {},
		grid
	}: {
		cla?: string;
		width?: string;
		height?: string;
		grid: Grid;
		pos: Position;
		onmousedown?: (e: MouseEvent) => void;
	} = $props();

	let cell = $derived(grid.getCell(pos));
	let active = $derived(grid.isActive(pos));
	let primaryactive = $derived(grid.isPrimaryActive(pos));
	let editing = $derived(grid.isEditing(pos));
	let externalediting = $derived(grid.isExternalEditing(pos));
	let width = $derived(grid.getColWidth(pos.col));
	let height = $derived(grid.getRowHeight(pos.row));
	let showPreview = $derived(getPreview() !== '');

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
			grid.stopEditing(pos);
			grid.resetCellTemp(pos);
		}
	}

	function getPreview() {
		return !isErr(cell?.temp_eval) ? getEvalLiteral(cell?.temp_eval) : '';
	}
</script>

{#if editing}
	<div class="relative inline-block">
		{#if showPreview}
			<h3
				class="bubble pointer-events-none absolute top-1/2 left-[2px] z-[500] -translate-y-[calc(50%+2.5em)] text-sm font-semibold tracking-tight text-foreground select-none"
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
				bind:value={() => cell?.temp_raw ?? '', (v) => grid.setCellTemp(pos, v)}
				onblur={() => {
					grid.stopEditing(cell?.pos);
					grid.setCell(cell?.pos);
				}}
			/>
		</div>
	</div>
{:else if cell && isErr(cell.eval)}
	<HoverCard.Root openDelay={500} closeDelay={100}>
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
		ondblclick={() => grid.startEditing(pos)}
		{onmousedown}
		data-row={pos.row}
		data-col={pos.col}
		ondragstart={(e) => e.preventDefault()}
		style:width
		style:height
		class={clsx(
			'placeholder bg-background p-1',
			{
				primaryactive,
				active,
				'active-top': grid.isActiveTop(pos),
				'active-bottom': grid.isActiveBottom(pos),
				'active-right': grid.isActiveRight(pos),
				'active-left': grid.isActiveLeft(pos),
				'only-active': grid.isActive(pos) && grid.isSingleActive()
			},
			cla
		)}
	>
		{#if cell && (cell.raw !== '' || getEvalLiteral(cell.eval) !== '')}
			<span
				class={clsx('pointer-events-none select-none', {
					err: isErr(cell.eval)
				})}
			>
				{#if externalediting}
					{cell.temp_raw}
				{:else if cell.eval}
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

	.primaryactive {
		z-index: 30 !important;
		border: 1px solid var(--color-primary) !important;
		outline: 1px solid var(--color-primary);
	}

	.active {
		z-index: 20;
		background-color: color-mix(in oklab, var(--color-primary) 20%, var(--color-background) 80%);
		border: 1px solid color-mix(in oklab, var(--input) 100%, var(--color-foreground) 5%);
		/* outline: 1px solid var(--color-primary); */
	}

	.only-active {
		background-color: transparent !important;
	}

	/* Borders for edges */
	.active-top {
		border-top: 1px solid var(--color-primary);
	}

	.active-bottom {
		border-bottom: 1px solid var(--color-primary);
	}

	.active-left {
		border-left: 1px solid var(--color-primary);
	}

	.active-right {
		border-right: 1px solid var(--color-primary);
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
