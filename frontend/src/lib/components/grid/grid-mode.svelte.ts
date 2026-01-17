import type { Grid } from './grid.svelte';
import type { Eval } from './messages';
import { Position } from './position.svelte';
import { GridSelection } from './grid-selection.svelte';
import {
	getVimId,
	registerVimListener,
	unregisterVimListener,
	vimKeyboardHandler,
	type VimCommand,
	type VimListener
} from './vim';

export type GridModeName = 'normal' | 'insert' | 'visual';

export class GridMode {
	#grid: Grid;

	constructor(grid: Grid) {
		this.#grid = grid;
	}

	// Used for initialization that requires DOM access
	public init() {}

	public destroy() {}

	public name(): GridModeName {
		return 'normal';
	}

	public getSelection(): GridSelection | null {
		return null;
	}

	public get grid(): Grid {
		return this.#grid;
	}
}

export class NormalMode extends GridMode implements VimListener {
	#selection: GridSelection = $state(new GridSelection());
	#vimId: string | null = null;

	constructor(grid: Grid, cellPos: Position) {
		super(grid);

		this.#selection = new GridSelection(cellPos, cellPos);
	}

	override init() {
		window.addEventListener('keydown', vimKeyboardHandler);
		this.#selection.init();

		this.#vimId = getVimId();
		registerVimListener(this.#vimId, this);
	}

	override getSelection(): GridSelection | null {
		return this.#selection;
	}

	override destroy() {
		window.removeEventListener('keydown', vimKeyboardHandler);
		this.#selection.destroy();

		if (this.#vimId) unregisterVimListener(this.#vimId);
	}

	onVimCommand(command: VimCommand): void {
		const shiftMap: Record<string, any> = {
			left: () => (this.#selection = this.#selection.shiftLeft(command.modifier)),
			down: () => (this.#selection = this.#selection.shiftDown(command.modifier)),
			up: () => (this.#selection = this.#selection.shiftUp(command.modifier)),
			right: () => (this.#selection = this.#selection.shiftRight(command.modifier))
		};

		const shift = (key: string) => {
			const fn = shiftMap[key];
			if (fn) {
				fn();
				this.#selection.init();
			}
		};

		if (command.action === 'visual-mode') {
			// Switch to visual mode
			this.destroy();
			this.grid.mode = new VisualMode(this.grid, this.#selection.primary);
			this.grid.mode.init();
			return;
		}

		if (!command.motion) return;
		shift(command.motion);
	}
}

export class InsertMode extends GridMode {
	#cellPos: Position;
	#preview: [Eval, boolean] | null = $state(null); // [Eval, dirty]

	constructor(grid: Grid, cellPos: Position) {
		super(grid);
		this.#cellPos = cellPos;
	}

	override name(): GridModeName {
		return 'insert';
	}

	override destroy() {}
}

export class VisualMode extends GridMode implements VimListener {
	#selection: GridSelection = $state(new GridSelection(new Position(0, 0), new Position(0, 0)));
	#vimId: string | null = null;

	constructor(grid: Grid, cellPos: Position) {
		super(grid);
		this.#selection = new GridSelection(cellPos, cellPos);
	}

	override init() {
		window.addEventListener('keydown', vimKeyboardHandler);
		this.#selection.init();

		this.#vimId = getVimId();
		registerVimListener(this.#vimId, this);
	}

	override name(): GridModeName {
		return 'visual';
	}

	override getSelection(): GridSelection | null {
		return this.#selection;
	}

	override destroy() {
		window.removeEventListener('keydown', vimKeyboardHandler);
		this.#selection.destroy();

		if (this.#vimId) unregisterVimListener(this.#vimId);
	}

	onVimCommand(command: VimCommand): void {
		const expandMap: Record<string, any> = {
			left: () => (this.#selection = this.#selection.expandLeft(command.modifier)),
			down: () => (this.#selection = this.#selection.expandDown(command.modifier)),
			up: () => (this.#selection = this.#selection.expandUp(command.modifier)),
			right: () => (this.#selection = this.#selection.expandRight(command.modifier))
		};

		const expand = (key: string) => {
			const fn = expandMap[key];
			if (fn) {
				fn();
				this.#selection.init();
			}
		};

		if (command.action === 'escape' || command.action === 'visual-mode') {
			// Switch to visual mode
			this.destroy();
			this.grid.mode = new NormalMode(this.grid, this.#selection.secondary);
			this.grid.mode.init();
			return;
		}

		if (!command.motion) return;
		expand(command.motion);
	}
}
