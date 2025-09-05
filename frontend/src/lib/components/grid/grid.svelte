<script lang="ts">
	import { toast } from 'svelte-sonner';
	import Cell from '$lib/components/grid/cell.svelte';
	import { onDestroy, onMount } from 'svelte';
	import CellHeader from './cell-header.svelte';
	import {
		refFromStr,
		splitErrorString,
		colToStr,
		refFromPos,
		type CellData,
		type CellValue
	} from './utils';

	let {
		socket
	}: {
		socket: WebSocket;
	} = $props();

	socket.onmessage = (msg: MessageEvent) => {
		const input = msg.data.toString().trim();

		if (input.startsWith('ERR')) {
			let split = splitErrorString(input);
			toast.error(split[0], {
				description: split[1]
			});

			return;
		}

		let strRef: string | undefined;
		let evalStr: string | undefined;

		// Case 1: "Cell D4 = Integer(4)"
		let match = input.match(/^Cell\s+([A-Z]+\d+)\s*=\s*(.+)$/);
		if (match) {
			[, strRef, evalStr] = match;
		}

		// Case 2: "D6 String("hello")" or "E9 Double(4.0)"
		if (!match) {
			match = input.match(/^([A-Z]+\d+)\s+(.+)$/);
			if (match) {
				[, strRef, evalStr] = match;
			}
		}

		if (!strRef || !evalStr) {
			console.warn('Unrecognized message:', input);
			return;
		}

		console.log(`Cell: ${strRef}`);
		console.log(`Eval: ${evalStr}`);

		let { row, col } = refFromStr(strRef);

		// Parse eval types
		if (evalStr.startsWith('Integer(')) {
			const num = parseInt(evalStr.match(/^Integer\(([-\d]+)\)$/)?.[1] ?? 'NaN', 10);
			console.log(`Parsed integer:`, num);
			setCellVal(row, col, num);
		} else if (evalStr.startsWith('Double(')) {
			const num = parseFloat(evalStr.match(/^Double\(([-\d.]+)\)$/)?.[1] ?? 'NaN');
			console.log(`Parsed double:`, num);
			setCellVal(row, col, num);
		} else if (evalStr.startsWith('String(')) {
			const str = evalStr.match(/^String\("(.+)"\)$/)?.[1];
			console.log(`Parsed string:`, str);
			setCellVal(row, col, str);
		}
	};

	let rows = 100;
	let cols = 60;

	let default_row_height = '30px';
	let default_col_width = '80px';

	// Only store touched cells
	let grid_vals: Record<string, CellData> = $state({});
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

	const getCell = (i: number, j: number) => grid_vals[key(i, j)] ?? undefined;

	const getCellRaw = (i: number, j: number) => getCell(i, j)?.raw_val ?? '';
	const setCellRaw = (i: number, j: number, val: string) => {
		if (grid_vals[key(i, j)] === undefined) {
			grid_vals[key(i, j)] = {
				raw_val: val,
				val: undefined
			};
		} else {
			grid_vals[key(i, j)].raw_val = val;
		}
	};
	const getCellVal = (i: number, j: number) => getCell(i, j)?.val ?? undefined;
	const setCellVal = (i: number, j: number, val: CellValue) => {
		if (grid_vals[key(i, j)] === undefined) {
			console.warn('Cell raw value was undefined but recieved eval.');
		} else {
			grid_vals[key(i, j)].val = val;
		}
	};

	const setCell = (i: number, j: number, v: string | undefined) => {
		// ignore “no value” so we don’t create keys on mount
		if (v == null || v === '') delete grid_vals[key(i, j)];
		else {
			setCellRaw(i, j, v);
			console.log(i, j);
			socket.send(`set ${refFromPos(i, j).str} ${v}`);
		}
	};

	// $effect(() => {
	// 	$inspect(grid_vals);
	// });

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
				const ref = refFromPos(i, j).str;
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

<div class="grid-wrapper">
	<div class="flex w-fit">
		<CellHeader height={default_row_height} width={default_col_width} val="" active={false} />
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
		<div class="flex w-fit">
			<CellHeader
				direction="row"
				width={default_col_width}
				height={getRowHeight(i)}
				setRowHeight={(height) => setRowHeight(i, height)}
				val={(i + 1).toString()}
				active={active_cell !== null && active_cell[0] === i}
			/>
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
					bind:raw_val={() => getCellRaw(i, j), (v) => setCell(i, j, v)}
					val={getCellVal(i, j)}
					active={active_cell !== null && active_cell[0] === i && active_cell[1] === j}
				/>
			{/each}
		</div>
	{/each}
</div>
