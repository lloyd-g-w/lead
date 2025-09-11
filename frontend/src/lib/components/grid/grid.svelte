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

	let dragging = $state(false);

	function handleCellMouseDown(i: number, j: number, e: MouseEvent) {
		let pos = new Position(i, j);

		if (grid.anyIsEditing()) {
			// Get the actual input element that's being edited
			const el = document.querySelector<HTMLInputElement>('input:focus');
			const currentInputValue = el?.value ?? '';

			// ONLY treat this as a reference insert if it's a formula
			if (currentInputValue.trim().startsWith('=')) {
				// Prevent the input from losing focus
				e.preventDefault();

				// --- This is the same reference-inserting logic as before ---
				const ref = refToStr(i, j);
				if (el) {
					const { selectionStart, selectionEnd } = el;
					const before = el.value.slice(0, selectionStart ?? 0);
					const after = el.value.slice(selectionEnd ?? 0);
					el.value = before + ref + after;
					const newPos = (selectionStart ?? 0) + ref.length;
					el.setSelectionRange(newPos, newPos);
					el.dispatchEvent(new Event('input', { bubbles: true }));
					el.focus();
				}

				return;
			}
		}

		// We are not editing, so this is a normal cell selection OR this is not a formula
		grid.setActive(pos, pos);
		dragging = true;
	}

	onMount(() => {
		const handler = (e: MouseEvent) => {
			// optional: check if click target is outside grid container
			if (!(e.target as HTMLElement).closest('.grid-wrapper')) {
				grid.stopEditingActive();
			}
		};
		window.addEventListener('click', handler);
		onDestroy(() => window.removeEventListener('click', handler));

		const handleMouseMove = (e: MouseEvent) => {
			if (!dragging) return;

			const el = document.elementFromPoint(e.clientX, e.clientY);

			if (el && el instanceof HTMLElement && el.dataset.row && el.dataset.col) {
				const row = parseInt(el.dataset.row, 10);
				const col = parseInt(el.dataset.col, 10);

				// expand selection as you drag
				console.log(`moved to ${refToStr(row, col)}`);
				grid.setActive(grid.primary_active, new Position(row, col));
			}
		};

		const handleMouseUp = () => {
			dragging = false; // stop tracking
		};

		window.addEventListener('mousemove', handleMouseMove);
		window.addEventListener('mouseup', handleMouseUp);

		onDestroy(() => {
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
						console.log(`down at ${refToStr(i, j)}`);
						handleCellMouseDown(i, j, e);
					}}
				/>
			{/each}
		</div>
	{/each}
</div>
