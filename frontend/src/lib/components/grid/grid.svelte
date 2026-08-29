<script lang="ts">
	import { EllipsisVertical } from '@lucide/svelte';
	import * as Alert from '$lib/components/ui/alert/index.js';
	import Cell from '$lib/components/grid/cell.svelte';
	import CellHeader from './cell-header.svelte';
	import { colToStr, getEvalLiteral, isErr, refToStr } from './utils';
	import clsx from 'clsx';
	import type { CellT, LeadMsg } from './messages';
import { Grid, Position } from './grid.svelte.ts';
import { GridSelection } from './grid-selection.svelte';
	import { onMount } from 'svelte';
	import { GRID_MODE_LABELS, InsertMode, NormalMode, VisualMode } from './grid-mode.svelte.ts';
	import ContextMenuSubTrigger from '../ui/context-menu/context-menu-sub-trigger.svelte';

	let {
		socket,
		class: className = '',
		wrapperEl = $bindable(undefined)
	}: {
		socket: WebSocket;
		class?: string;
		wrapperEl?: HTMLElement;
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
	let mouseSelecting = false;
	let mouseDragMoved = false;
	let mouseAnchor: Position | null = null;
	let mouseLastPos: Position | null = null;

	let visibleRowsStart = $state(0); // Could also be considered as offset
	let visibleRowsEnd = $derived.by(() => {
		if (!wrapperEl) return rows;
		return grid.getLastRowAfterPx(visibleRowsStart, vh);
	});
	let visibleColsStart = $state(0);
	let visibleColsEnd = $derived.by(() => {
		if (!wrapperEl) return cols;
		return grid.getLastColAfterPx(visibleColsStart, vw);
	});

	let vw = $state(0);
	let vh = $state(0);

	function updateViewport() {
		if (wrapperEl) {
			vw = wrapperEl.clientWidth;
			vh = wrapperEl.clientHeight;
		} else {
			vw = window.innerWidth;
			vh = window.innerHeight;
		}

		visibleRowsEnd = grid.getLastRowAfterPx(visibleRowsStart, vh);
		visibleColsEnd = grid.getLastColAfterPx(visibleColsStart, vw);

		console.log('Viewport updated:', {
			visibleRowsStart,
			visibleRowsEnd,
			visibleColsStart,
			visibleColsEnd
		});
	}

	$effect(() => {
		if (!wrapperEl) return;
		const resizeObserver = new ResizeObserver(() => {
			updateViewport();
		});

		resizeObserver.observe(wrapperEl);

		return () => resizeObserver.disconnect();
	});

	const selectionBox = $derived(grid.mode.getSelection());
	const modeName = $derived(grid.mode.name());
	const modeLabel = $derived(GRID_MODE_LABELS[modeName]);
	const insertSelection = $derived(grid.mode instanceof InsertMode ? grid.mode.getSelection() : null);
	const insertPos = $derived(insertSelection?.primary ?? null);
	const insertCell = $derived(insertPos ? grid.getCell(insertPos) : null);

	$effect(() => {
		if (!selectionBox || !wrapperEl) return;
		console.log(
			'selectionbox primary px top + primaryPxHeight:',
			selectionBox.primaryPxTop + selectionBox.primaryPxHeight
		);
		console.log('wrapper bottom:', wrapperEl.getBoundingClientRect().bottom);

		if (
			selectionBox.primaryPxTop + selectionBox.primaryPxHeight >
			wrapperEl?.getBoundingClientRect().bottom - 100
		) {
			visibleRowsStart = grid.getRowAfterPx(
				selectionBox.primary.row,
				wrapperEl.clientHeight - selectionBox.primaryPxHeight - 50
			);
		}
	});

	// Formula state
	let selectingRangeForFormula = false;
	let anchorRow = -1,
		anchorCol = -1,
		hoverRow = -1,
		hoverCol = -1;
	let formulaCaretStart: number | null = null,
		formulaCaretEnd: number | null = null;
	let insertRangeSelecting = $state(false);
	let insertRangeAnchor: Position | null = $state(null);
	let insertRangeToken: HTMLSpanElement | null = $state(null);
	let insertRangeTarget: Position | null = $state(null);
	let insertRangeSelection: GridSelection | null = $state(null);
	let insertRangeSpacer: Text | null = $state(null);
	let insertCaretRange: Range | null = $state(null);
	let insertCaretMarker: HTMLSpanElement | null = $state(null);
	let lastInsertKey: string | null = null;

	const overlaySelection = $derived(
		insertRangeSelecting && insertRangeSelection ? insertRangeSelection : selectionBox
	);

function rangeRef(r1: number, c1: number, r2: number, c2: number) {
	const rs = Math.min(r1, r2),
		re = Math.max(r1, r2);
	const cs = Math.min(c1, c2),
		ce = Math.max(c1, c2);
	const a = refToStr(rs, cs),
		b = refToStr(re, ce);
	return rs === re && cs === ce ? a : `${a}:${b}`;
}

function placeCaretAtEnd(element: HTMLElement) {
	const selection = window.getSelection();
	if (!selection) return;
	const range = document.createRange();
	range.selectNodeContents(element);
	range.collapse(false);
	selection.removeAllRanges();
	selection.addRange(range);
}

	function placeCaretAfterNode(node: Node) {
		const selection = window.getSelection();
		if (!selection) return;
		const range = document.createRange();
		range.setStartAfter(node);
		range.collapse(true);
		selection.removeAllRanges();
		selection.addRange(range);
	}

	function setCaretAfterNode(editor: HTMLElement, node: Node) {
		const selection = window.getSelection();
		if (!selection) return;
		const range = document.createRange();
		range.setStartAfter(node);
		range.collapse(true);
		selection.removeAllRanges();
		selection.addRange(range);
		insertCaretRange = range.cloneRange();
	}

function updateInsertValue(pos: Position) {
	if (!(grid.mode instanceof InsertMode)) return;
	const editor = grid.mode.insertEl;
	if (!editor) return;
	grid.setCellTemp(pos, editor.innerText);
}

	function ensureInsertCaret(editor: HTMLElement) {
		const selection = window.getSelection();
		if (!selection || selection.rangeCount === 0) {
			placeCaretAtEnd(editor);
			return;
		}
		const anchor = selection.anchorNode;
		if (anchor && !editor.contains(anchor)) {
			placeCaretAtEnd(editor);
		}
	}

	function captureInsertCaret(editor: HTMLElement) {
		const selection = window.getSelection();
		if (!selection || selection.rangeCount === 0) return;
		const range = selection.getRangeAt(0);
		if (!editor.contains(range.startContainer)) return;
		insertCaretRange = range.cloneRange();
	}

	function restoreInsertCaret(editor: HTMLElement) {
		if (!insertCaretRange) return;
		if (!editor.contains(insertCaretRange.startContainer)) return;
		const selection = window.getSelection();
		if (!selection) return;
		selection.removeAllRanges();
		selection.addRange(insertCaretRange);
	}

	function placeInsertCaretMarker(editor: HTMLElement) {
		const selection = window.getSelection();
		if (!selection || selection.rangeCount === 0) return;
		const range = selection.getRangeAt(0).cloneRange();
		if (!editor.contains(range.startContainer)) return;
		const marker = document.createElement('span');
		marker.className = 'caret-marker';
		marker.contentEditable = 'false';
		range.collapse(false);
		insertCaretMarker?.remove();
		range.insertNode(marker);
		insertCaretMarker = marker;
		placeCaretAfterNode(marker);
	}

	function setInsertRangeSelection(anchor: Position, target: Position) {
		insertRangeSelection?.destroy();
		insertRangeSelection = new GridSelection(anchor, target);
		insertRangeSelection.init();
	}

	function clearInsertRangeSelection() {
		insertRangeSelection?.destroy();
		insertRangeSelection = null;
	}

	function insertTokenAtCaret(editor: HTMLElement, token: HTMLSpanElement) {
		const marker = insertCaretMarker;
		if (marker && editor.contains(marker)) {
			const spacer = document.createTextNode(' ');
			marker.replaceWith(token, spacer);
			insertRangeSpacer = spacer;
			insertCaretMarker = null;
			queueMicrotask(() => placeCaretAfterNode(spacer));
			return;
		}

		let targetRange = insertCaretRange;
		if (!targetRange || !editor.contains(targetRange.startContainer)) {
			const fallback = document.createRange();
			fallback.selectNodeContents(editor);
			fallback.collapse(false);
			targetRange = fallback;
		}

		targetRange.deleteContents();
		const spacer = document.createTextNode(' ');
		const frag = document.createDocumentFragment();
		frag.appendChild(token);
		frag.appendChild(spacer);
		targetRange.insertNode(frag);
		insertRangeSpacer = spacer;
		queueMicrotask(() => placeCaretAfterNode(spacer));
	}

	function forceCaretAfterNode(editor: HTMLElement, node: Node) {
		editor.focus();
		requestAnimationFrame(() => {
			setCaretAfterNode(editor, node);
			requestAnimationFrame(() => setCaretAfterNode(editor, node));
		});
	}

	function startInsertRangeSelection(anchor: Position) {
		if (!(grid.mode instanceof InsertMode)) return;
		const editor = grid.mode.insertEl;
		const targetPos = grid.mode.getSelection()?.primary ?? null;
		if (!editor || !targetPos) return;

		editor.focus();
		restoreInsertCaret(editor);
		ensureInsertCaret(editor);

		const token = document.createElement('span');
		token.className = 'range-token';
		token.contentEditable = 'false';
		token.textContent = anchor.str();

		insertTokenAtCaret(editor, token);
		if (insertRangeSpacer) {
			forceCaretAfterNode(editor, insertRangeSpacer);
		}

		insertRangeSelecting = true;
		insertRangeAnchor = anchor;
		insertRangeToken = token;
		insertRangeTarget = targetPos;
		setInsertRangeSelection(anchor, anchor);
		updateInsertValue(targetPos);
	}

	function updateInsertRangeToken(target: Position) {
		if (!insertRangeToken || !insertRangeAnchor || !insertRangeTarget) return;
		insertRangeToken.textContent = rangeRef(
			insertRangeAnchor.row,
			insertRangeAnchor.col,
			target.row,
			target.col
		);
		const caretTarget = insertRangeSpacer ?? insertRangeToken;
		queueMicrotask(() => placeCaretAfterNode(caretTarget));
		updateInsertValue(insertRangeTarget);
	}

	function setNormalSelection(pos: Position) {
		grid.setErrorTooltip(null);
		if (grid.mode instanceof NormalMode) {
			grid.mode.updateSelection(pos);
			return;
		}
		if (grid.mode instanceof InsertMode) {
			grid.mode.submitCell();
		}
		grid.mode.destroy();
		grid.mode = new NormalMode(grid, pos);
		grid.mode.init();
	}

	function setVisualSelection(anchor: Position, target: Position) {
		grid.setErrorTooltip(null);
		if (grid.mode instanceof VisualMode) {
			grid.mode.updateSelection(anchor, target);
			return;
		}
		if (grid.mode instanceof InsertMode) {
			grid.mode.submitCell();
		}
		grid.mode.destroy();
		const visualMode = new VisualMode(grid, anchor);
		visualMode.init();
		visualMode.updateSelection(anchor, target);
		grid.mode = visualMode;
	}

	function setInsertMode(pos: Position) {
		grid.setErrorTooltip(null);
		if (grid.mode instanceof InsertMode) {
			const current = grid.mode.getSelection()?.primary;
			if (!current?.equals(pos)) {
				grid.mode.submitCell();
			}
			grid.mode.updateSelection(pos);
			return;
		}
		grid.mode.destroy();
		grid.mode = new InsertMode(grid, pos);
		grid.mode.init();
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
		if (grid.mode instanceof InsertMode) {
			e.preventDefault();
			if (grid.mode.insertEl) {
				grid.mode.insertEl.focus();
				captureInsertCaret(grid.mode.insertEl);
			}
			startInsertRangeSelection(new Position(i, j));
			dragging = true;
			return;
		}
	e.preventDefault();
	grid.stopAnyEditing();
	const pos = new Position(i, j);
		setNormalSelection(pos);
		mouseSelecting = true;
		mouseDragMoved = false;
		mouseAnchor = pos;
		mouseLastPos = pos;
		dragging = true;
	}

	function keepInsertFocus(e: PointerEvent) {
		if (!(grid.mode instanceof InsertMode)) return;
		const editor = grid.mode.insertEl;
		if (!editor) return;
		if (e.target instanceof Node && editor.contains(e.target)) return;
		placeInsertCaretMarker(editor);
	}

	function handleCellDoubleClick(i: number, j: number) {
		setInsertMode(new Position(i, j));
	}

	onMount(() => {
		const handleMouseMove = (e: MouseEvent) => {
			if (!dragging) return;
			const el = document.elementFromPoint(e.clientX, e.clientY);
			if (el instanceof HTMLElement && el.dataset.row && el.dataset.col) {
				const row = parseInt(el.dataset.row, 10);
				const col = parseInt(el.dataset.col, 10);
				hoverRow = row;
				hoverCol = col;
				if (insertRangeSelecting) {
					const nextPos = new Position(row, col);
					updateInsertRangeToken(nextPos);
					if (insertRangeAnchor) {
						setInsertRangeSelection(insertRangeAnchor, nextPos);
					}
				}
				if (!selectingRangeForFormula && mouseSelecting && mouseAnchor) {
					const nextPos = new Position(row, col);
					mouseLastPos = nextPos;
					if (!mouseAnchor.equals(nextPos)) {
						mouseDragMoved = true;
					}
					if (mouseDragMoved) {
						setVisualSelection(mouseAnchor, nextPos);
					}
				}
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
			if (insertRangeSelecting) {
				const editor = grid.mode instanceof InsertMode ? grid.mode.insertEl : null;
				const caretTarget = insertRangeSpacer ?? insertRangeToken;
				if (editor && caretTarget) {
					forceCaretAfterNode(editor, caretTarget);
				}
				insertRangeSelecting = false;
				insertRangeAnchor = null;
				insertRangeToken = null;
				insertRangeTarget = null;
				insertRangeSpacer = null;
				clearInsertRangeSelection();
			}
			if (mouseSelecting && mouseAnchor) {
				if (!mouseDragMoved) {
					setNormalSelection(mouseAnchor);
				} else if (mouseLastPos) {
					setVisualSelection(mouseAnchor, mouseLastPos);
				}
			}
			mouseSelecting = false;
			mouseDragMoved = false;
			mouseAnchor = null;
			mouseLastPos = null;
			dragging = false;
		};

		window.addEventListener('mousemove', handleMouseMove);
		window.addEventListener('mouseup', handleMouseUp);
		const handleSelectionChange = () => {
			if (!(grid.mode instanceof InsertMode)) return;
			const editor = grid.mode.insertEl;
			if (!editor) return;
			captureInsertCaret(editor);
		};
		document.addEventListener('selectionchange', handleSelectionChange);
		return () => {
			window.removeEventListener('mousemove', handleMouseMove);
			window.removeEventListener('mouseup', handleMouseUp);
			document.removeEventListener('selectionchange', handleSelectionChange);
		};
	});

	$effect(() => {
		if (!(grid.mode instanceof InsertMode)) {
			lastInsertKey = null;
			return;
		}
		const editor = grid.mode.insertEl;
		if (!editor || !insertPos) return;
		const key = insertPos.key();
		if (key !== lastInsertKey) {
			editor.innerText = insertCell?.temp_raw ?? insertCell?.raw ?? '';
			lastInsertKey = key;
		}
	});

	function getInsertModePreview(cell: CellT | undefined | null) {
		if (!cell) return '';
		return !isErr(cell?.temp_eval) ? getEvalLiteral(cell?.temp_eval) : '';
	}

	function range(start: number, end: number, step = 1) {
		const output = [];
		for (let i = start; i <= end; i += step) {
			output.push(i);
		}
		return output;
	}
</script>

<div class="relative mb-5 ml-5 flex items-center gap-[5px]">
	<Alert.Root
		class="flex h-6 w-fit min-w-[70px] items-center justify-center rounded-md border bg-input/30 px-2 text-xs font-semibold uppercase tracking-wide shadow-xs"
	>
		{modeLabel} mode
	</Alert.Root>
	<Alert.Root
		class="flex h-6 w-fit min-w-[70px] items-center justify-center rounded-md border bg-input/30 px-2 text-xs shadow-xs"
	>
		{grid.mode?.getSelection()?.toString() ?? ''}
	</Alert.Root>
	<EllipsisVertical class="text-muted-foreground" size="18px" />
</div>

<div
	class={clsx(
		'grid-wrapper relative h-full w-full overflow-hidden text-xs outline-none',
		className
	)}
	onpointerdowncapture={keepInsertFocus}
>
	<div class="relative w-fit">
		<div class="sticky top-0 flex w-fit" style="z-index: 100;">
			<div class="sticky left-0 z-[101]">
				<CellHeader
					direction="blank"
					height={grid.getDefaultRowHeight()}
					width={grid.getDefaultColWidth()}
					defaultHeight={grid.getDefaultRowHeight()}
					defaultWidth={grid.getDefaultColWidth()}
					val=""
					active={false}
				/>
			</div>
			{#each range(visibleColsStart, visibleColsEnd) as j}
				<CellHeader
					direction="col"
					height={grid.getDefaultRowHeight()}
					width={grid.getColWidth(j)}
					defaultHeight={grid.getDefaultRowHeight()}
					defaultWidth={grid.getDefaultColWidth()}
					setColWidth={(w) => grid.setColWidth(j, w)}
					val={colToStr(j)}
					active={grid.mode?.getSelection()?.containsCol(j) ?? false}
				/>
			{/each}
		</div>

		{#each range(visibleRowsStart, visibleRowsEnd) as i}
			<div class="flex">
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
				{#each range(visibleColsStart, visibleColsEnd) as j}
					{@const pos = new Position(i, j)}
					{@const cell = grid.getCell(pos)}
					<Cell
						{grid}
						{pos}
						height={grid.getRowHeight(i)}
						width={grid.getColWidth(j)}
						showErrorTooltip={!!grid.errorTooltipPos?.equals(pos) && isErr(cell?.eval)}
						onmousedown={(e) => handleCellMouseDown(i, j, e)}
						ondblclick={() => handleCellDoubleClick(i, j)}
					/>
				{/each}
			</div>
		{/each}

		{#if grid.mode instanceof InsertMode && grid.mode.getSelection()}
			{@const insertMode = grid.mode as InsertMode}
			{@const insertBox = grid.mode.getSelection()!}
			{@const pos = insertBox.primary}
			{@const cell = grid.getCell(insertBox.primary)}
			{@const preview = getInsertModePreview(cell)}

			<!-- Background to hide the text underneath -->
			<div
				class="pointer-events-none absolute z-35 bg-background"
				style:top="{insertBox.primaryPxTop}px"
				style:left="{insertBox.primaryPxLeft}px"
				style:width="{insertBox.primaryPxWidth}px"
				style:height="{insertBox.primaryPxHeight}px"
			></div>

			{#if preview !== undefined && preview !== '' && preview !== null}
				<h3
					class="bubble pointer-events-none absolute z-[500] w-fit -translate-y-[calc(50%+0.6em)] text-xs font-semibold text-foreground select-none"
					role="tooltip"
					style:top="{insertBox.primaryPxTop}px"
					style:left="{insertBox.primaryPxLeft}px"
				>
					{preview}
				</h3>
			{/if}

			<div
				onfocus={() => {
					if (insertMode.insertEl) {
						restoreInsertCaret(insertMode.insertEl);
						captureInsertCaret(insertMode.insertEl);
					}
				}}
				contenteditable="true"
				class="absolute z-[600] h-fit w-fit max-w-md border-2 border-primary bg-background px-1 py-0.5 text-[0.65rem] text-wrap break-words ring-2 ring-primary/30 outline-none focus:outline-none"
				style:top="{insertBox.primaryPxTop}px"
				style:left="{insertBox.primaryPxLeft}px"
				style:min-width="{insertBox.primaryPxWidth}px"
				style:min-height="{insertBox.primaryPxHeight}px"
				bind:this={insertMode.insertEl}
				oninput={() => {
					updateInsertValue(pos);
					if (insertMode.insertEl) {
						captureInsertCaret(insertMode.insertEl);
					}
				}}
				onkeydown={() => {
					if (insertMode.insertEl) {
						captureInsertCaret(insertMode.insertEl);
					}
				}}
				onkeyup={() => {
					if (insertMode.insertEl) {
						captureInsertCaret(insertMode.insertEl);
					}
				}}
				onmouseup={() => {
					if (insertMode.insertEl) {
						captureInsertCaret(insertMode.insertEl);
					}
				}}
				autofocus
			></div>
		{/if}

		{#if overlaySelection && (!(grid.mode instanceof InsertMode) || insertRangeSelecting)}
			{@const isSingle = overlaySelection.isSingleCell()}
			<div class="pointer-events-none absolute inset-0 z-40">
				<div
					class="absolute border-2 border-primary"
					style:top="{overlaySelection.primaryPxTop}px"
					style:left="{overlaySelection.primaryPxLeft}px"
					style:width="{overlaySelection.primaryPxWidth}px"
					style:height="{overlaySelection.primaryPxHeight}px"
				></div>

				{#if !isSingle}
					<div
						class="absolute border-2 border-dashed border-primary"
						style:top="{overlaySelection.secondaryPxTop}px"
						style:left="{overlaySelection.secondaryPxLeft}px"
						style:width="{overlaySelection.secondaryPxWidth}px"
						style:height="{overlaySelection.secondaryPxHeight}px"
					></div>
					<div
						class="absolute border border-primary bg-primary/20"
						style:top="{overlaySelection.pxTop}px"
						style:left="{overlaySelection.pxLeft}px"
						style:width="{overlaySelection.pxWidth}px"
						style:height="{overlaySelection.pxHeight}px"
					></div>
				{/if}
			</div>
		{/if}
	</div>
</div>

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
		border-radius: 4px;
		color: var(--color-popover-foreground);
		margin: auto;
		box-shadow: 0 2px 18px rgba(0, 0, 0, 0.08);
		max-width: min(15rem, 20vw);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		padding: 0.15rem 0.3rem;
		display: flex;
		align-items: center;
		line-height: 1.2;
	}

	.range-token {
		display: inline-flex;
		align-items: center;
		gap: 0.15rem;
		padding: 0.05rem 0.35rem;
		border-radius: 999px;
		border: 1px solid color-mix(in oklab, var(--color-primary) 50%, white 50%);
		background: color-mix(in oklab, var(--color-primary) 12%, white 88%);
		color: var(--color-primary);
		font-weight: 600;
		font-size: 0.7em;
		letter-spacing: 0.02em;
		vertical-align: middle;
		white-space: nowrap;
		user-select: none;
	}

	.caret-marker {
		display: inline-block;
		width: 0;
		height: 0;
		overflow: hidden;
	}


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
