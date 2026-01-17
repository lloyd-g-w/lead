import { Position } from './position.svelte';
import type { VimModifier } from './vim';

export class GridSelection {
	#p1: Position; // This is the position that initated the selection
	#p2: Position; // This is the position that ended the selection

	#tl: Position; // Top-left of box
	#br: Position; // You can guess

	#pxTop: number = $state(0); // The pixel position of the top-left corner
	#pxLeft: number = $state(0);
	#pxWidth: number = $state(0); // The pixel dimensions of the selection
	#pxHeight: number = $state(0);

	#primaryPxTop: number = $state(0); // The pixel position of the primary selection cell
	#primaryPxLeft: number = $state(0);
	#primaryPxWidth: number = $state(0);
	#primaryPxHeight: number = $state(0);

	#secondaryPxTop: number = $state(0); // The pixel position of the secondary selection cell
	#secondaryPxLeft: number = $state(0);
	#secondaryPxWidth: number = $state(0);
	#secondaryPxHeight: number = $state(0);

	#observer: MutationObserver | null = null;

	#updatePxDimensions() {
		const tlEl = document.querySelector(
			`[data-row="${this.#tl.row}"][data-col="${this.#tl.col}"]`
		) as HTMLElement;
		const brEl = document.querySelector(
			`[data-row="${this.#br.row}"][data-col="${this.#br.col}"]`
		) as HTMLElement;
		const p1El = document.querySelector(
			`[data-row="${this.#p1.row}"][data-col="${this.#p1.col}"]`
		) as HTMLElement;
		const p2El = document.querySelector(
			`[data-row="${this.#p2.row}"][data-col="${this.#p2.col}"]`
		) as HTMLElement;

		if (!tlEl || !brEl || !p1El || !p2El) return;

		// Use offsetLeft/offsetTop relative to the scrolling container
		// This assumes the cells are positioned relative to the grid-wrapper
		this.#pxLeft = tlEl.offsetLeft;
		this.#pxTop = tlEl.offsetTop;
		this.#pxWidth = brEl.offsetLeft + brEl.offsetWidth - tlEl.offsetLeft;
		this.#pxHeight = brEl.offsetTop + brEl.offsetHeight - tlEl.offsetTop;

		this.#primaryPxLeft = p1El.offsetLeft;
		this.#primaryPxTop = p1El.offsetTop;
		this.#primaryPxWidth = p1El.offsetWidth;
		this.#primaryPxHeight = p1El.offsetHeight;

		this.#secondaryPxLeft = p2El.offsetLeft;
		this.#secondaryPxTop = p2El.offsetTop;
		this.#secondaryPxWidth = p2El.offsetWidth;
		this.#secondaryPxHeight = p2El.offsetHeight;
	}

	constructor(p1: Position = new Position(), p2: Position = new Position()) {
		this.#p1 = p1;
		this.#p2 = p2;

		this.#tl = new Position(
			Math.min(this.#p1.row, this.#p2.row),
			Math.min(this.#p1.col, this.#p2.col)
		);

		this.#br = new Position(
			Math.max(this.#p1.row, this.#p2.row),
			Math.max(this.#p1.col, this.#p2.col)
		);
	}

	// For dom-related initialization
	public init() {
		this.#updatePxDimensions();

		this.#observer = new MutationObserver(() => {
			this.#updatePxDimensions();
		});
		// On dom change, update pixel dimensions
		// We use a mutation observer to detect changes in the DOM that might affect the selection dimensions

		this.#observer.observe(document.body, {
			attributes: true,
			childList: true,
			subtree: true
		});
	}

	public destroy() {
		this.#observer?.disconnect();
	}

	public get tl(): Position {
		return this.#tl;
	}

	public get br(): Position {
		return this.#br;
	}

	public get primary(): Position {
		return this.#p1;
	}

	public get secondary(): Position {
		return this.#p2;
	}

	public get pxTop(): number {
		return this.#pxTop;
	}

	public get pxLeft(): number {
		return this.#pxLeft;
	}

	public get pxWidth(): number {
		return this.#pxWidth;
	}

	public get pxHeight(): number {
		return this.#pxHeight;
	}

	public get primaryPxTop(): number {
		return this.#primaryPxTop;
	}
	public get primaryPxLeft(): number {
		return this.#primaryPxLeft;
	}
	public get primaryPxWidth(): number {
		return this.#primaryPxWidth;
	}
	public get primaryPxHeight(): number {
		return this.#primaryPxHeight;
	}

	public get secondaryPxTop(): number {
		return this.#secondaryPxTop;
	}
	public get secondaryPxLeft(): number {
		return this.#secondaryPxLeft;
	}
	public get secondaryPxWidth(): number {
		return this.#secondaryPxWidth;
	}
	public get secondaryPxHeight(): number {
		return this.#secondaryPxHeight;
	}

	public contains(pos: Position) {
		return (
			pos.row >= this.#tl.row &&
			pos.row <= this.#br.row &&
			pos.col >= this.#tl.col &&
			pos.col <= this.#br.col
		);
	}

	public containsRow(row: number) {
		return row >= this.#tl.row && row <= this.#br.row;
	}

	public containsCol(col: number) {
		return col >= this.#tl.col && col <= this.#br.col;
	}

	public isSingleCell(): boolean {
		return this.#tl.equals(this.#br);
	}

	public expandUp(amount: VimModifier = 1) {
		return new GridSelection(
			this.#p1,
			new Position(Math.max(0, this.#p2.row - amount), this.#p2.col)
		);
	}

	public expandDown(amount: VimModifier = 1) {
		return new GridSelection(this.#p1, new Position(this.#p2.row + amount, this.#p2.col));
	}

	public expandLeft(amount: VimModifier = 1) {
		return new GridSelection(
			this.#p1,
			new Position(this.#p2.row, Math.max(0, this.#p2.col - amount))
		);
	}

	public expandRight(amount: VimModifier = 1) {
		return new GridSelection(this.#p1, new Position(this.#p2.row, this.#p2.col + amount));
	}

	public shiftUp(amount: VimModifier = 1) {
		return new GridSelection(
			new Position(Math.max(0, this.#p1.row - amount), this.#p1.col),
			new Position(Math.max(0, this.#p2.row - amount), this.#p2.col)
		);
	}

	public shiftDown(amount: VimModifier = 1) {
		return new GridSelection(
			new Position(this.#p1.row + amount, this.#p1.col),
			new Position(this.#p2.row + amount, this.#p2.col)
		);
	}

	public shiftLeft(amount: VimModifier = 1) {
		return new GridSelection(
			new Position(this.#p1.row, Math.max(0, this.#p1.col - amount)),
			new Position(this.#p2.row, Math.max(0, this.#p2.col - amount))
		);
	}

	public shiftRight(amount: VimModifier = 1) {
		return new GridSelection(
			new Position(this.#p1.row, this.#p1.col + amount),
			new Position(this.#p2.row, this.#p2.col + amount)
		);
	}
}
