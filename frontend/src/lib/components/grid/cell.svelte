<script lang="ts">
	import { Input } from '$lib/components/ui/input/index.js';
	import clsx from 'clsx';
	import { getErrDesc, getErrTitle, getEvalLiteral, isErr, type CellT } from './utils';
	import * as HoverCard from '$lib/components/ui/hover-card/index.js';

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
				el.value = cell?.raw_val ?? '';
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
			bind:value={
				() => {
					return cell?.raw_val ?? '';
				},
				(v) => {
					cell = {
						val: cell?.val,
						raw_val: v
					};
				}
			}
			onblur={(e) => {
				// cell = {
				// 	val: cell?.val,
				// 	raw_val: (e.target as HTMLInputElement).value
				// };
				stopediting();
			}}
		/>
	</div>
{:else if cell && isErr(cell.val)}
	<HoverCard.Root openDelay={500} closeDelay={500}>
		<HoverCard.Trigger>
			{@render InnerCell()}
		</HoverCard.Trigger>
		<HoverCard.Content side="right">
			<h2 class="text-md font-semibold tracking-tight transition-colors">
				{getErrTitle(cell.val)}
			</h2>
			{getErrDesc(cell.val)}
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
		{#if cell && (cell.raw_val !== '' || getEvalLiteral(cell.val) !== '')}
			<span class={clsx('pointer-events-none select-none', { err: isErr(cell.val) })}>
				{#if cell.val && !externalediting}
					{getEvalLiteral(cell.val)}
				{:else}
					{cell.raw_val}
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
</style>
