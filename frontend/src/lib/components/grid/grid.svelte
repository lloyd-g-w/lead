<script lang="ts">
	import { EllipsisVertical, Omega } from '@lucide/svelte';
	import * as Alert from '$lib/components/ui/alert/index.js';
	import Cell from '$lib/components/grid/cell.svelte';
	import CellHeader from './cell-header.svelte';
	import { colToStr, refToStr } from './utils';
	import clsx from 'clsx';
	import { Input } from '../ui/input';
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

	socket.onmessage = (msg: MessageEvent) => {
		let res: LeadMsg;

		try {
			res = JSON.parse(msg.data);
			console.log(res);
		} catch (err) {
			console.error('Failed to parse LeadMsg:', err);
			return;
		}

		grid.handle_msg(res);
	};

	const grid = $state(new Grid(socket));
	let rows = 100;
	let cols = 50;
	// --- module-level state ------------------------------------------------------
	let dragging = false;

	// range picking while editing a formula:
	let selectingRangeForFormula = false;
	let anchorRow = -1;
	let anchorCol = -1;
	let hoverRow = -1;
	let hoverCol = -1;
	let formulaCaretStart: number | null = null;
	let formulaCaretEnd: number | null = null;

	// helper: A1 or A1:B9 using your existing refToStr()
	function rangeRef(r1: number, c1: number, r2: number, c2: number) {
		const rs = Math.min(r1, r2);
		const re = Math.max(r1, r2);
		const cs = Math.min(c1, c2);
		const ce = Math.max(c1, c2);

		const a = refToStr(rs, cs);
		const b = refToStr(re, ce);
		return rs === re && cs === ce ? a : `${a}:${b}`;
	}

	// --- your existing handler, modified ----------------------------------------
	function handleCellMouseDown(i: number, j: number, e: MouseEvent) {
		const pos = new Position(i, j);

		if (grid.anyIsEditing()) {
			const el = document.querySelector<HTMLInputElement>('input:focus');
			const currentInputValue = el?.value ?? '';

			// Only treat as reference insert if we're editing a formula
			if (currentInputValue.trim().startsWith('=')) {
				// Keep focus in the input
				e.preventDefault();

				// Enter "select range for formula" mode, but DO NOT insert yet.
				selectingRangeForFormula = true;
				dragging = true;

				anchorRow = i;
				anchorCol = j;
				hoverRow = i;
				hoverCol = j;

				// remember the caret where we'll insert the reference on mouseup
				if (el) {
					formulaCaretStart = el.selectionStart ?? el.value.length;
					formulaCaretEnd = el.selectionEnd ?? el.value.length;
					el.focus();
				} else {
					formulaCaretStart = formulaCaretEnd = null;
				}

				// visually highlight the starting cell
				grid.setActive(new Position(anchorRow, anchorCol), new Position(anchorRow, anchorCol));
				return;
			}

			// Not a formula; exit editing before doing a normal selection
			grid.stopAnyEditing();
		}

		// Normal (non-formula) selection behavior
		grid.setActive(pos, pos);
		dragging = true;
	}

	onMount(() => {
		const handler = (e: MouseEvent) => {
			// If click is outside the grid, cancel editing
			if (!(e.target as HTMLElement).closest('.grid-wrapper')) {
				grid.stopEditingActive();

				// also reset any in-progress formula selection
				selectingRangeForFormula = false;
				dragging = false;
			}
		};

		const handleMouseMove = (e: MouseEvent) => {
			if (!dragging) return;

			const el = document.elementFromPoint(e.clientX, e.clientY);
			if (el && el instanceof HTMLElement && el.dataset.row && el.dataset.col) {
				const row = parseInt(el.dataset.row, 10);
				const col = parseInt(el.dataset.col, 10);

				hoverRow = row;
				hoverCol = col;

				if (selectingRangeForFormula) {
					// while dragging a formula range, keep the grid selection in sync
					grid.setActive(new Position(anchorRow, anchorCol), new Position(row, col));
				} else {
					// normal drag-select
					grid.setActive(grid.primary_active, new Position(row, col));
				}
			}
		};

		const handleMouseUp = () => {
			// Commit the range to the formula input iff we were range-picking
			if (selectingRangeForFormula) {
				const input = document.querySelector<HTMLInputElement>('input:focus');

				// Fallbacks in case caret wasn't captured (shouldn't happen if input stayed focused)
				const start = formulaCaretStart ?? input?.value.length ?? 0;
				const end = formulaCaretEnd ?? start;

				const ref = rangeRef(anchorRow, anchorCol, hoverRow, hoverCol);

				if (input) {
					const before = input.value.slice(0, start);
					const after = input.value.slice(end);
					input.value = before + ref + after;

					const newPos = start + ref.length;
					input.setSelectionRange(newPos, newPos);
					input.dispatchEvent(new Event('input', { bubbles: true }));
					input.focus();
				}

				// Reset formula-range state but keep the user in edit mode
				selectingRangeForFormula = false;
				grid.clearActive();
			}

			dragging = false;

			// clear transient state
			anchorRow = anchorCol = hoverRow = hoverCol = -1;
			formulaCaretStart = formulaCaretEnd = null;
		};

		window.addEventListener('click', handler);
		window.addEventListener('mousemove', handleMouseMove);
		window.addEventListener('mouseup', handleMouseUp);

		onDestroy(() => {
			window.removeEventListener('click', handler);
			window.removeEventListener('mousemove', handleMouseMove);
			window.removeEventListener('mouseup', handleMouseUp);
		});
	});
</script>

<div class="relative mb-5 ml-5 flex items-center gap-[5px]">
	<Alert.Root
		class={clsx(
			'flex h-9 w-fit min-w-[80px] rounded-md border border-input bg-transparent px-2 text-sm font-medium shadow-xs ring-offset-background transition-[color,box-shadow] outline-none selection:bg-primary selection:text-primary-foreground placeholder:text-muted-foreground disabled:cursor-not-allowed disabled:opacity-50 md:text-sm dark:bg-input/30',
			'focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50',
			'flex items-center justify-center aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40'
		)}
	>
		{grid.getActiveRangeStr()}
	</Alert.Root>

	<EllipsisVertical class="text-muted-foreground" size="20px" />

	<div
		class="relative"
		onkeydown={(e: KeyboardEvent) => {
			const target = e.currentTarget as HTMLElement;
			const input = target.querySelector('input') as HTMLInputElement | null;

			if (e.key === 'Enter' || e.key === 'NumpadEnter') {
				e.preventDefault(); // avoid form submit/line break

				grid.stopExternalEdit(grid.getActivePos());
				grid.setCell(grid.getActivePos());
				input?.blur();
			} else if (e.key == 'Escape') {
				e.preventDefault();

				grid.stopExternalEdit(grid.getActivePos());
				grid.resetCellTemp(grid.getActivePos());
				input?.blur();
			}
		}}
	>
		<Omega
			size="20px"
			class="absolute top-1/2 left-2 z-10 -translate-y-1/2 text-muted-foreground"
		/>
		<Input
			disabled={grid.getActivePos() === null}
			onmousedown={() => grid.startExternalEdit(grid.getActivePos())}
			onblur={() => grid.stopExternalEdit(grid.getActivePos())}
			bind:value={
				() => grid.getActiveCell()?.temp_raw ?? '',
				(v) => {
					grid.setCellTemp(grid.getActivePos(), v);
				}
			}
			class="relative w-fit min-w-[300px] pl-9"
		></Input>
	</div>
</div>

<div
	class={clsx(' grid-wrapper relative h-full min-h-0 max-w-full min-w-0 overflow-auto', className)}
>
	<div class="sticky top-0 flex w-fit" style="z-index: {rows + 70}">
		<div class="sticky top-0 left-0" style="z-index: {rows + 70}">
			<CellHeader
				resizeable={false}
				height={grid.getDefaultRowHeight()}
				width={grid.getDefaultColWidth()}
				val=""
				active={false}
				direction="blank"
			/>
		</div>

		{#each Array(cols) as _, j}
			<CellHeader
				height={grid.getDefaultRowHeight()}
				width={grid.getColWidth(j)}
				setColWidth={(width) => grid.setColWidth(j, width)}
				direction="col"
				val={colToStr(j)}
				active={grid.primary_active !== null &&
					grid.secondary_active !== null &&
					j >= Math.min(grid.primary_active.col, grid.secondary_active.col) &&
					j <= Math.max(grid.primary_active.col, grid.secondary_active.col)}
			/>
		{/each}
	</div>
	{#each Array(rows) as _, i}
		<div class="relative flex w-fit">
			<div class="sticky left-0 flex w-fit" style="z-index: {rows - i + 40}">
				<CellHeader
					direction="row"
					height={grid.getRowHeight(i)}
					width={grid.getDefaultColWidth()}
					setRowHeight={(height) => grid.setRowHeight(i, height)}
					val={(i + 1).toString()}
					active={grid.primary_active !== null &&
						grid.secondary_active !== null &&
						i >= Math.min(grid.primary_active.row, grid.secondary_active.row) &&
						i <= Math.max(grid.primary_active.row, grid.secondary_active.row)}
				/>
			</div>
			{#each Array(cols) as _, j}
				<Cell
					{grid}
					pos={new Position(i, j)}
					height={grid.getRowHeight(i)}
					width={grid.getColWidth(j)}
					onmousedown={(e) => {
						handleCellMouseDown(i, j, e);
					}}
				/>
			{/each}
		</div>
	{/each}
</div>
