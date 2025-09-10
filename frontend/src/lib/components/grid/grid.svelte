<script lang="ts">
	import { Omega } from '@lucide/svelte';
	import Cell from '$lib/components/grid/cell.svelte';
	import { onMount } from 'svelte';
	import CellHeader from './cell-header.svelte';
	import { colToStr, refToStr } from './utils';
	import clsx from 'clsx';
	import { Input } from '../ui/input';
	import type { LeadMsg } from './messages';
	import { Grid, Position } from './grid.svelte.ts';

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

	const grid = new Grid(socket);
	let rows = 10;
	let cols = 10;

	function handleCellInteraction(i: number, j: number, e: MouseEvent) {
		let pos = new Position(i, j);

		if (grid.isEditing(pos)) {
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
		grid.setActive(pos);
	}

	onMount(() => {
		// const handler = (e: MouseEvent) => {
		// optional: check if click target is outside grid container
		// if (!(e.target as HTMLElement).closest('.grid-wrapper')) {
		// 	active_cell = null;
		// }
		// };
		// window.addEventListener('click', handler);
		// onDestroy(() => window.removeEventListener('click', handler));
	});
</script>

<div class="relative mb-5 ml-5 flex items-center gap-[5px]">
	<div
		class="relative"
		onkeydown={(e: KeyboardEvent) => {
			if (e.key === 'Enter' || e.key === 'NumpadEnter') {
				e.preventDefault(); // avoid form submit/line break
				const el = (e.currentTarget as HTMLElement).querySelector(
					'input'
				) as HTMLInputElement | null;
				el?.blur(); // triggers on:blur below
			} else if (e.key == 'Escape') {
				e.preventDefault();

				grid.stopEditingActive();
			}
		}}
	>
		<Omega
			size="20px"
			class="absolute top-1/2 left-2 z-10 -translate-y-1/2 text-muted-foreground"
		/>
		<Input
			onmousedown={() => grid.setExternalEdit(grid.getActivePos())}
			onblur={() => grid.setCell(grid.getActivePos())}
			bind:value={
				() => grid.getActiveCell()?.temp_raw ?? '',
				(v) => {
					grid.setCellTemp(grid.getActivePos(), v);
				}
			}
			class="relative w-[200px] pl-9"
		></Input>
	</div>
</div>

<div
	class={clsx(
		' grid-wrapper relative h-full min-h-0 max-w-full min-w-0 overflow-auto overflow-visible',
		className
	)}
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
				active={grid.getActivePos() !== null && grid.getActivePos()?.col === j}
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
					active={grid.getActivePos() !== null && grid.getActivePos()?.row === i}
				/>
			</div>
			{#each Array(cols) as _, j}
				<Cell
					height={grid.getRowHeight(i)}
					width={grid.getColWidth(j)}
					editing={grid.isEditing(new Position(i, j))}
					externalediting={grid.isExternalEditing(new Position(i, j))}
					startediting={() => grid.startEditing(new Position(i, j))}
					stopediting={() => grid.stopEditing(new Position(i, j))}
					onmousedown={(e) => {
						handleCellInteraction(i, j, e);
					}}
					bind:cell={
						() => grid.getCell(new Position(i, j)),
						(v) => grid.setCellTemp(new Position(i, j), v?.temp_raw)
					}
					active={grid.isActive(new Position(i, j))}
				/>
			{/each}
		</div>
	{/each}
</div>
