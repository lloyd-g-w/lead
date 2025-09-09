<script lang="ts">
	import { toast } from 'svelte-sonner';
	import { Pencil } from '@lucide/svelte';
	import Cell from '$lib/components/grid/cell.svelte';
	import { onDestroy, onMount } from 'svelte';
	import CellHeader from './cell-header.svelte';
	import { colToStr, refToStr, type CellT } from './utils';
	import clsx from 'clsx';
	import { Input } from '../ui/input';

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

		handle_msg(res);
	};

	function handle_msg(msg: LeadMsg) {
		switch (msg.msg_type) {
			case 'error': {
				toast.error('Error', {
					description: msg.raw
				});
				break;
			}
			case 'set': {
				if (msg.cell === undefined) {
					console.error('Expected cell ref for SET msgponse from server.');
					return;
				} else if (msg.eval === undefined) {
					console.error('Expected cell value for SET msgponse from server.');
					return;
				}

				grid_vals[key(msg.cell.row, msg.cell.col)] = {
					raw_val: msg.raw ?? '',
					val: msg.eval
				};

				break;
			}
			case 'bulk': {
				if (msg.bulk_msgs === undefined) {
					console.error('Expected bulk_msgs field to be defined for BULK message.');
					return;
				}

				for (const m of msg.bulk_msgs) handle_msg(m);
			}
		}
	}

	let rows = 50;
	let cols = 40;

	let default_row_height = '30px';
	let default_col_width = '80px';

	// Only store touched cells
	let grid_vals: Record<string, CellT> = $state({});
	let row_heights: Record<number, string> = $state({});
	let col_widths: Record<number, string> = $state({});

	function getRowHeight(row: number) {
		return row_heights[row] ?? default_row_height;
	}

	function getColWidth(col: number) {
		return col_widths[col] ?? default_col_width;
	}

	function setRowHeight(row: number, height: string) {
		if (height === default_row_height) {
			delete row_heights[row];
		} else {
			row_heights[row] = height;
		}
	}

	function setColWidth(col: number, width: string) {
		if (width === default_col_width) {
			delete col_widths[col];
		} else {
			col_widths[col] = width;
		}
	}

	let active_cell: [number, number] | null = $state(null);
	let editing_cell: [number, number] | null = $state(null);

	function startEditing(i: number, j: number) {
		active_cell = [i, j];
		editing_cell = [i, j];
	}

	function stopEditing() {
		editing_cell = null;
	}

	const key = (i: number, j: number) => `${i}:${j}`;

	const getCell = (i: number, j: number) => grid_vals[key(i, j)];
	const setCell = (row: number, col: number, v: CellT) => {
		if (v?.raw_val == null || v.raw_val === '') {
			delete grid_vals[key(row, col)];
			return;
		}

		grid_vals[key(row, col)] = {
			raw_val: v.raw_val,
			val: v.val
		};

		let msg: LeadMsg = {
			msg_type: 'set',
			cell: { row, col },
			raw: v.raw_val
		};

		socket.send(JSON.stringify(msg));
	};

	function handleCellInteraction(i: number, j: number, e: MouseEvent) {
		if (editing_cell) {
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
		active_cell = [i, j];
	}

	function getActiveCell(): CellT {
		if (active_cell != null && grid_vals[key(active_cell[0], active_cell[1])])
			return grid_vals[key(active_cell[0], active_cell[1])];
		else
			return {
				raw_val: '',
				val: undefined
			};
	}

	function setActiveCellRaw(raw: string): void {
		if (active_cell == null) return;

		if (grid_vals[key(active_cell[0], active_cell[1])]) {
			let cell = grid_vals[key(active_cell[0], active_cell[1])];
			setCell(active_cell[0], active_cell[1], {
				raw_val: raw,
				val: cell.val
			});
		} else {
			setCell(active_cell[0], active_cell[1], {
				raw_val: raw,
				val: undefined
			});
		}
	}

	onMount(() => {
		const handler = (e: MouseEvent) => {
			// optional: check if click target is outside grid container
			if (!(e.target as HTMLElement).closest('.grid-wrapper')) {
				active_cell = null;
			}
		};
		window.addEventListener('click', handler);
		onDestroy(() => window.removeEventListener('click', handler));
	});
</script>

<div class="mb-5 ml-5 flex items-center gap-5">
	<Pencil />
	<Input
		bind:value={() => getActiveCell().raw_val, (raw) => setActiveCellRaw(raw)}
		class="relative w-[200px] rounded-none p-1 !transition-none delay-0 duration-0
			focus:z-20 focus:shadow-[0_0_0_1px_var(--color-primary)] focus:outline-none"
	></Input>
</div>

<div
	class={clsx('grid-wrapper relative h-full min-h-0 max-w-full min-w-0 overflow-auto', className)}
>
	<div class="sticky top-0 flex w-fit" style="z-index: {rows + 70}">
		<div class="sticky top-0 left-0" style="z-index: {rows + 70}">
			<CellHeader
				resizeable={false}
				height={default_row_height}
				width={default_col_width}
				val=""
				active={false}
				direction="blank"
			/>
		</div>

		{#each Array(cols) as _, j}
			<CellHeader
				height={default_row_height}
				width={getColWidth(j)}
				setColWidth={(width) => setColWidth(j, width)}
				direction="col"
				val={colToStr(j)}
				active={active_cell !== null && active_cell[1] === j}
			/>
		{/each}
	</div>
	{#each Array(rows) as _, i}
		<div class="relative flex w-fit">
			<div class="sticky left-0 flex w-fit" style="z-index: {rows - i + 40}">
				<CellHeader
					direction="row"
					width={default_col_width}
					height={getRowHeight(i)}
					setRowHeight={(height) => setRowHeight(i, height)}
					val={(i + 1).toString()}
					active={active_cell !== null && active_cell[0] === i}
				/>
			</div>
			{#each Array(cols) as _, j}
				<Cell
					height={getRowHeight(i)}
					width={getColWidth(j)}
					editing={editing_cell?.[0] === i && editing_cell?.[1] === j}
					startediting={() => startEditing(i, j)}
					stopediting={stopEditing}
					onmousedown={(e) => {
						handleCellInteraction(i, j, e);
					}}
					bind:cell={() => getCell(i, j), (v) => setCell(i, j, v)}
					active={active_cell !== null && active_cell[0] === i && active_cell[1] === j}
				/>
			{/each}
		</div>
	{/each}
</div>
