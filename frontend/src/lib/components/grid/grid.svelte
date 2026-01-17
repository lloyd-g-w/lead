<script lang="ts">
	import { EllipsisVertical } from '@lucide/svelte';
	import * as Alert from '$lib/components/ui/alert/index.js';
	import Cell from '$lib/components/grid/cell.svelte';
	import CellHeader from './cell-header.svelte';
	import { colToStr, refToStr } from './utils';
	import clsx from 'clsx';
	import type { LeadMsg } from './messages';
	import { Grid, Position } from './grid.svelte.ts';
	import { onDestroy, onMount } from 'svelte';

	let {
		socket,
		class: className = ''
	}: {
		class?: string;
		socket: WebSocket;
	} = $props();

	const grid = $state(new Grid(socket));

	socket.onmessage = (msg: MessageEvent) => {
		try {
			const res: LeadMsg = JSON.parse(msg.data);
			grid.handle_msg(res);
		} catch (err) {
			console.error('Failed to parse LeadMsg:', err);
		}
	};

	onMount(() => {
		grid.init();
	});

	let rows = 100;
	let cols = 50;
	let dragging = false;

	const selectionBox = $derived(grid.mode.getSelection());

	// Formula state
	let selectingRangeForFormula = false;
	let anchorRow = -1,
		anchorCol = -1,
		hoverRow = -1,
		hoverCol = -1;
	let formulaCaretStart: number | null = null,
		formulaCaretEnd: number | null = null;

	function rangeRef(r1: number, c1: number, r2: number, c2: number) {
		const rs = Math.min(r1, r2),
			re = Math.max(r1, r2);
		const cs = Math.min(c1, c2),
			ce = Math.max(c1, c2);
		const a = refToStr(rs, cs),
			b = refToStr(re, ce);
		return rs === re && cs === ce ? a : `${a}:${b}`;
	}

	function handleCellMouseDown(i: number, j: number, e: MouseEvent) {
		const input = document.querySelector<HTMLInputElement>('input:focus');
		if (input?.value.trim().startsWith('=')) {
			e.preventDefault();
			selectingRangeForFormula = true;
			dragging = true;
			anchorRow = hoverRow = i;
			anchorCol = hoverCol = j;
			formulaCaretStart = input.selectionStart ?? input.value.length;
			formulaCaretEnd = input.selectionEnd ?? input.value.length;
			return;
		}
		grid.stopAnyEditing();
		dragging = true;
	}

	onMount(() => {
		const handleMouseMove = (e: MouseEvent) => {
			if (!dragging) return;
			const el = document.elementFromPoint(e.clientX, e.clientY);
			if (el instanceof HTMLElement && el.dataset.row && el.dataset.col) {
				hoverRow = parseInt(el.dataset.row, 10);
				hoverCol = parseInt(el.dataset.col, 10);
			}
		};

		const handleMouseUp = () => {
			if (selectingRangeForFormula) {
				const input = document.querySelector<HTMLInputElement>('input:focus');
				if (input) {
					const ref = rangeRef(anchorRow, anchorCol, hoverRow, hoverCol);
					const start = formulaCaretStart ?? 0,
						end = formulaCaretEnd ?? start;
					input.value = input.value.slice(0, start) + ref + input.value.slice(end);
					input.dispatchEvent(new Event('input', { bubbles: true }));
				}
				selectingRangeForFormula = false;
			}
			dragging = false;
		};

		window.addEventListener('mousemove', handleMouseMove);
		window.addEventListener('mouseup', handleMouseUp);
		return () => {
			window.removeEventListener('mousemove', handleMouseMove);
			window.removeEventListener('mouseup', handleMouseUp);
		};
	});
</script>

<div class="relative mb-5 ml-5 flex items-center gap-[5px]">
	<Alert.Root class="h-9 w-fit min-w-[80px] border bg-input/30 px-2 text-xs shadow-xs"></Alert.Root>
	<EllipsisVertical class="text-muted-foreground" size="20px" />
</div>

<div class={clsx('grid-wrapper relative h-full overflow-auto text-xs outline-none', className)}>
	<div class="relative w-fit min-w-full">
		<div class="sticky top-0 flex w-fit" style="z-index: 100;">
			<div class="sticky left-0 z-[101]">
				<CellHeader
					direction="blank"
					height={grid.getDefaultRowHeight()}
					width={grid.getDefaultColWidth()}
					val=""
					active={false}
				/>
			</div>
			{#each Array(cols) as _, j}
				<CellHeader
					direction="col"
					height={grid.getDefaultRowHeight()}
					width={grid.getColWidth(j)}
					setColWidth={(w) => grid.setColWidth(j, w)}
					val={colToStr(j)}
					active={grid.mode?.getSelection()?.containsCol(j) ?? false}
				/>
			{/each}
		</div>

		{#each Array(rows) as _, i}
			<div class="flex w-fit">
				<div class="sticky left-0 z-50 flex w-fit">
					<CellHeader
						direction="row"
						height={grid.getRowHeight(i)}
						width={grid.getDefaultColWidth()}
						setRowHeight={(h) => grid.setRowHeight(i, h)}
						val={(i + 1).toString()}
						active={grid.mode?.getSelection()?.containsRow(i) ?? false}
					/>
				</div>
				{#each Array(cols) as _, j}
					<Cell
						{grid}
						pos={new Position(i, j)}
						height={grid.getRowHeight(i)}
						width={grid.getColWidth(j)}
						onmousedown={(e) => handleCellMouseDown(i, j, e)}
					/>
				{/each}
			</div>
		{/each}

		{#if selectionBox}
			{@const isSingle = selectionBox.isSingleCell()}
			<div class="pointer-events-none absolute inset-0 z-40">
				<div
					class="absolute border-2 border-primary"
					style:top="{selectionBox.primaryPxTop}px"
					style:left="{selectionBox.primaryPxLeft}px"
					style:width="{selectionBox.primaryPxWidth}px"
					style:height="{selectionBox.primaryPxHeight}px"
				></div>

				{#if !isSingle}
					<div
						class="absolute border-2 border-dashed border-primary"
						style:top="{selectionBox.secondaryPxTop}px"
						style:left="{selectionBox.secondaryPxLeft}px"
						style:width="{selectionBox.secondaryPxWidth}px"
						style:height="{selectionBox.secondaryPxHeight}px"
					></div>
					<div
						class="absolute bg-primary/20"
						style:top="{selectionBox.pxTop}px"
						style:left="{selectionBox.pxLeft}px"
						style:width="{selectionBox.pxWidth}px"
						style:height="{selectionBox.pxHeight}px"
					></div>
				{/if}
			</div>
		{/if}
	</div>
</div>
