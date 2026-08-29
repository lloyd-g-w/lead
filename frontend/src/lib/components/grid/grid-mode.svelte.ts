import type { Grid } from './grid.svelte.ts';
import { Position } from './position.svelte';
import { GridSelection } from './grid-selection.svelte';
import {
	getVimId,
	registerVimListener,
	unregisterVimListener,
	vimInsertModeKeyboardHandler,
	vimNormalModeKeyboardHandler,
	type VimCommand,
	type VimListener
} from './vim';

export type GridModeName = 'normal' | 'insert' | 'visual';

export const GRID_MODE_LABELS: Record<GridModeName, string> = {
	normal: 'Normal',
	insert: 'Insert',
	visual: 'Visual'
};

export class GridMode {
	#grid: Grid;
	#initalised: boolean = $state(false);

	constructor(grid: Grid) {
		this.#grid = grid;
	}

	// Used for initialization that requires DOM access
	public init() {
		this.#initalised = true;
	}

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

	public get initalised(): boolean {
		return this.#initalised;
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
		super.init();

		this.#selection.init();

		window.addEventListener('keydown', vimNormalModeKeyboardHandler);
		this.#vimId = getVimId();
		registerVimListener(this.#vimId, this);
	}

	override getSelection(): GridSelection | null {
		if (!this.initalised) return null;
		return this.#selection;
	}

	public updateSelection(pos: Position): void {
		this.#selection.destroy();
		this.#selection = new GridSelection(pos, pos);
		if (this.initalised) this.#selection.init();
	}

	override destroy() {
		window.removeEventListener('keydown', vimNormalModeKeyboardHandler);
		if (this.#vimId) unregisterVimListener(this.#vimId);

		this.#selection.destroy();
	}

	onVimCommand(command: VimCommand): void {
		if (command.action !== 'show-error') {
			this.grid.setErrorTooltip(null);
		}
		const shiftMap: Record<string, any> = {
			down: () => (this.#selection = this.#selection.shiftDown(command.modifier)),
			left: () => (this.#selection = this.#selection.shiftLeft(command.modifier)),
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

		switch (command.action) {
			case 'visual-mode':
				this.destroy();
				this.grid.mode = new VisualMode(this.grid, this.#selection.primary);
				this.grid.mode.init();
				break;

			case 'insert-mode':
				this.destroy();
				this.grid.mode = new InsertMode(this.grid, this.#selection.primary);
				this.grid.mode.init();
				break;

			case 'delete':
				this.grid.clearCell(this.#selection.primary);
				break;

			case 'goto':
				if (command.motion === 'top') {
					(this.#selection = this.#selection.shiftTop()).init();
				} else if (command.motion === 'start') {
					(this.#selection = this.#selection.shiftStart()).init();
				}
				break;
			case 'show-error':
				this.grid.setErrorTooltip(this.#selection.primary);
				break;

			default:
				break;
		}

		if (!command.motion) return;
		shift(command.motion);
	}
}

export class VisualMode extends GridMode implements VimListener {
	#selection: GridSelection = $state(new GridSelection(new Position(0, 0), new Position(0, 0)));
	#vimId: string | null = null;

	constructor(grid: Grid, cellPos: Position) {
		super(grid);
		this.#selection = new GridSelection(cellPos, cellPos);
	}

	override init() {
		super.init();

		window.addEventListener('keydown', vimNormalModeKeyboardHandler);
		this.#selection.init();

		this.#vimId = getVimId();
		registerVimListener(this.#vimId, this);
	}

	override name(): GridModeName {
		return 'visual';
	}

	override getSelection(): GridSelection | null {
		if (!this.initalised) return null;
		return this.#selection;
	}

	public updateSelection(primary: Position, secondary: Position): void {
		this.#selection.destroy();
		this.#selection = new GridSelection(primary, secondary);
		if (this.initalised) this.#selection.init();
	}

	override destroy() {
		window.removeEventListener('keydown', vimNormalModeKeyboardHandler);
		this.#selection.destroy();

		if (this.#vimId) unregisterVimListener(this.#vimId);
	}

	onVimCommand(command: VimCommand): void {
		if (command.action !== 'show-error') {
			this.grid.setErrorTooltip(null);
		}
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

		switch (command.action) {
			case 'escape':
			case 'visual-mode':
				this.destroy();
				this.grid.mode = new NormalMode(this.grid, this.#selection.secondary);
				this.grid.mode.init();
				return;
			case 'delete':
				this.#selection.forEach((pos: Position) => {
					this.grid.clearCell(pos);
				});
				return;

			case 'goto':
				if (command.motion === 'top') {
					(this.#selection = this.#selection.expandTop()).init();
				} else if (command.motion === 'start') {
					(this.#selection = this.#selection.expandStart()).init();
				}
				return;
			case 'show-error':
				this.grid.setErrorTooltip(this.#selection.primary);
				return;
			default:
				break;
		}

		if (!command.motion) return;
		expand(command.motion);
	}
}

export class InsertMode extends GridMode implements VimListener {
	#selection: GridSelection = $state(new GridSelection());
	#vimId: string | null = null;
	#insertEl: HTMLElement | null = $state(null);

	constructor(grid: Grid, cellPos: Position) {
		super(grid);

		this.#selection = new GridSelection(cellPos, cellPos);
	}

	override name(): GridModeName {
		return 'insert';
	}

	override destroy() {
		this.#selection.destroy();

		window.removeEventListener('keydown', vimInsertModeKeyboardHandler);
		if (this.#vimId) unregisterVimListener(this.#vimId);
	}

	override init() {
		super.init();

		this.#selection.init();

		this.#vimId = getVimId();
		registerVimListener(this.#vimId, this);
		window.addEventListener('keydown', vimInsertModeKeyboardHandler);
	}

	override getSelection(): GridSelection | null {
		if (!this.initalised) return null;
		return this.#selection;
	}

	public updateSelection(pos: Position): void {
		this.#selection.destroy();
		this.#selection = new GridSelection(pos, pos);
		if (this.initalised) this.#selection.init();
	}

	public onVimCommand(command: VimCommand): void {
		const switchToNormalMode = () => {
			this.destroy();
			this.grid.mode = new NormalMode(this.grid, this.#selection.primary);
			this.grid.mode.init();
		};

		switch (command.action) {
			case 'escape':
				this.resetCellTemp();
				switchToNormalMode();
				return;

			case 'submit':
				this.submitCell();
				switchToNormalMode();
				return;

			default:
				return;
		}
	}

	public get insertEl(): HTMLElement | null {
		return this.#insertEl;
	}

	public set insertEl(el: HTMLElement | null) {
		this.#insertEl = el;
	}

	public resetCellTemp(): void {
		this.grid.resetCellTemp(this.#selection.primary);
	}

	public submitCell(): void {
		this.grid.setCell(this.#selection.primary);
	}
}
