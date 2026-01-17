import { toast } from 'svelte-sonner';
import type { CellT, Eval, LeadMsg } from './messages';
import type { GridData, GridDefaults } from './types';
import { NormalMode, type GridMode } from './grid-mode.svelte';
import { Position } from './position.svelte';

class Grid {
	socket: WebSocket;

	mode: GridMode = $state(new NormalMode(this, new Position(0, 0)));

	defaults: GridDefaults;

	data: GridData = $state({ cells: {}, row_heights: {}, col_widths: {} });

	constructor(socket: WebSocket, defaults: GridDefaults = { row_height: 20, col_width: 80 }) {
		this.socket = socket;
		this.defaults = defaults;
	}

	init() {
		this.mode.init();
	}

	public getCell(pos: Position): CellT | undefined {
		return this.data.cells[pos.key()];
	}

	public setCell(pos: Position | null | undefined) {
		if (pos === null || pos === undefined) return;
		let cell = this.data.cells[pos.key()];
		if (cell === undefined) return;

		if (cell.temp_raw === '') {
			delete this.data.cells[pos.key()];
			return;
		}

		cell.raw = cell.temp_raw;
		cell.eval = cell.temp_eval;

		let msg: LeadMsg = {
			msg_type: 'set',
			cell: pos.ref(),
			raw: cell.temp_raw
		};

		this.socket.send(JSON.stringify(msg));
	}

	public setCellTemp(pos: Position | null, raw: string | undefined) {
		if (pos === null || raw === undefined) return;

		let x = this.data.cells[pos.key()];

		this.data.cells[pos.key()] = {
			raw: x?.raw ?? '',
			temp_raw: raw,
			pos: pos,
			eval: x?.eval ?? undefined,
			temp_eval: x?.temp_eval ?? undefined
		};

		this.quickEval(pos, raw);
	}

	public resetCellTemp(pos: Position | null | undefined) {
		if (!pos) return;

		let x = this.data.cells[pos.key()];

		this.data.cells[pos.key()] = {
			raw: x?.raw ?? '',
			pos: pos,
			temp_raw: x?.raw ?? '',
			eval: x?.eval ?? undefined,
			temp_eval: undefined
		};
	}

	// public clearActive() {
	// 	this.setActive(null, null);
	// }

	public getRowHeight(row: number) {
		return this.data.row_heights[row] ?? this.defaults.row_height;
	}

	public getColWidth(col: number) {
		return this.data.col_widths[col] ?? this.defaults.col_width;
	}

	public setColWidth(col: number, width: number) {
		if (width === this.defaults.col_width) {
			delete this.data.col_widths[col];
		} else {
			this.data.col_widths[col] = width;
		}
	}

	public setRowHeight(row: number, height: number) {
		if (height === this.defaults.row_height) {
			delete this.data.row_heights[row];
		} else {
			this.data.row_heights[row] = height;
		}
	}

	public getDefaultColWidth() {
		return this.defaults.col_width;
	}
	public getDefaultRowHeight() {
		return this.defaults.row_height;
	}

	// public isActiveTop(pos: Position): boolean {
	// 	const tl = this.getActiveTopLeft();
	// 	if (!tl) return false;
	// 	return this.isActive(pos) && pos.row === tl.row;
	// }
	//
	// public isActiveBottom(pos: Position): boolean {
	// 	const br = this.getActiveBottomRight();
	// 	if (!br) return false;
	// 	return this.isActive(pos) && pos.row === br.row;
	// }
	//
	// public isActiveLeft(pos: Position): boolean {
	// 	const tl = this.getActiveTopLeft();
	// 	if (!tl) return false;
	// 	return this.isActive(pos) && pos.col === tl.col;
	// }
	//
	// public isActiveRight(pos: Position): boolean {
	// 	const br = this.getActiveBottomRight();
	// 	if (!br) return false;
	// 	return this.isActive(pos) && pos.col === br.col;
	// }
	//
	// public setRowHeight(row: number, height: string) {
	// 	if (height === this.defaults.row_height) {
	// 		delete this.data.row_heights[row];
	// 	} else {
	// 		this.data.row_heights[row] = height;
	// 	}
	// }
	//
	// public setColWidth(col: number, width: string) {
	// 	if (width === this.defaults.col_width) {
	// 		delete this.data.col_widths[col];
	// 	} else {
	// 		this.data.col_widths[col] = width;
	// 	}
	// }
	//
	// public startEditing(pos: Position | undefined) {
	// 	if (!pos) return;
	//
	// 	this.setActive(pos, pos);
	// 	// this.editing_cell = pos;
	//
	// 	let cell = this.getCell(pos);
	// 	if (!cell) return;
	// }

	public stopEditing(pos: Position | null | undefined) {
		if (!pos) return;
		// this.editing_cell = null;
		// this.setCell(pos);
	}

	public stopAnyEditing() {
		// this.editing_cell = null;
	}

	// public stopEditingActive() {
	// 	if (!this.anyIsActive() || !this.primary_active?.equals(this.secondary_active)) return;
	// 	this.stopEditing(this.primary_active);
	// }
	//
	// public isEditing(pos: Position): boolean {
	// 	if (this.editing_cell === null) return false;
	// 	return this.editing_cell.equals(pos);
	// }
	//
	// public anyIsEditing(): boolean {
	// 	return this.editing_cell !== null;
	// }
	//
	// public isExternalEditing(pos: Position): boolean {
	// 	if (this.external_editing_cell === null) return false;
	// 	return this.external_editing_cell.equals(pos);
	// }
	//
	// public setActive(primary: Position | null, secondary: Position | null) {
	// 	this.primary_active = primary;
	// 	this.secondary_active = secondary;
	// }
	//
	// public setInactive() {
	// 	this.primary_active = null;
	// 	this.secondary_active = null;
	// }
	//
	// public startExternalEdit(pos: Position | null) {
	// 	this.external_editing_cell = pos;
	// }
	//
	// public stopExternalEdit(pos: Position | null) {
	// 	this.external_editing_cell = null;
	// }
	//
	// public getActiveCell(): CellT | undefined {
	// 	if (this.primary_active === null || this.secondary_active === null) {
	// 		return {
	// 			raw: '',
	// 			temp_raw: '',
	// 			pos: new Position(-1, -1),
	// 			eval: undefined
	// 		};
	// 	}
	//
	// 	if (!this.primary_active.equals(this.secondary_active)) {
	// 		return {
	// 			raw: '',
	// 			temp_raw: '',
	// 			pos: new Position(-1, -1),
	// 			eval: undefined
	// 		};
	// 	}
	//
	// 	return this.getCell(this.primary_active);
	// }
	//
	// public getActiveRangeStr(): string {
	// 	const tl = this.getActiveTopLeft();
	// 	const br = this.getActiveBottomRight();
	//
	// 	if (tl === null || br === null) return '';
	//
	// 	// Single-cell selection
	// 	if (tl.equals(br)) return tl.str();
	//
	// 	// Range selection
	// 	return `${tl.str()}:${br.str()}`;
	// }
	//
	// public getActivePos(): Position | null {
	// 	if (
	// 		this.primary_active === null ||
	// 		this.secondary_active === null ||
	// 		!this.primary_active.equals(this.secondary_active)
	// 	) {
	// 		return null;
	// 	}
	// 	return this.primary_active;
	// }

	// public isActive(pos: Position): boolean {
	// 	if (this.primary_active === null || this.secondary_active === null) return false;
	//
	// 	return (
	// 		pos.row >= Math.min(this.primary_active.row, this.secondary_active.row) &&
	// 		pos.row <= Math.max(this.primary_active.row, this.secondary_active.row) &&
	// 		pos.col >= Math.min(this.primary_active.col, this.secondary_active.col) &&
	// 		pos.col <= Math.max(this.primary_active.col, this.secondary_active.col)
	// 	);
	// }

	// public getActiveTopLeft(): Position | null {
	// 	if (this.primary_active === null || this.secondary_active === null) return null;
	//
	// 	return new Position(
	// 		Math.min(this.primary_active.row, this.secondary_active.row),
	// 		Math.min(this.primary_active.col, this.secondary_active.col)
	// 	);
	// }

	// public getActiveBottomRight(): Position | null {
	// 	if (this.primary_active === null || this.secondary_active === null) return null;
	//
	// 	return new Position(
	// 		Math.max(this.primary_active.row, this.secondary_active.row),
	// 		Math.max(this.primary_active.col, this.secondary_active.col)
	// 	);
	// }

	// public isPrimaryActive(pos: Position): boolean {
	// 	if (this.primary_active === null) return false;
	// 	return this.primary_active.equals(pos);
	// }
	//
	// public isSingleActive(): boolean {
	// 	return this.getActivePos() !== null;
	// }
	//
	// public anyIsActive(): boolean {
	// 	return this.primary_active !== null && this.secondary_active !== null;
	// }

	public quickEval(pos: Position | null, raw: string) {
		if (pos === null) return;

		let msg: LeadMsg = {
			msg_type: 'eval',
			cell: pos.ref(),
			raw: raw
		};

		this.socket.send(JSON.stringify(msg));
	}

	public handle_msg(msg: LeadMsg) {
		switch (msg.msg_type) {
			case 'error': {
				toast.error('Error', {
					description: msg.raw
				});
				break;
			}
			case 'set': {
				if (msg.cell === undefined) {
					console.error('Expected cell ref for SET msg from server.');
					return;
				} else if (msg.eval === undefined) {
					console.error('Expected cell value for SET msg from server.');
					return;
				}

				let pos = new Position(msg.cell.row, msg.cell.col);

				let x = this.data.cells[pos.key()];

				this.data.cells[pos.key()] = {
					raw: msg.raw ?? '',
					eval: msg.eval,
					pos: pos,
					temp_raw: x?.temp_raw ?? '',
					temp_eval: x?.temp_eval ?? undefined
				};

				break;
			}
			case 'bulk': {
				if (msg.bulk_msgs === undefined) {
					console.error('Expected bulk_msgs field to be defined for BULK message.');
					return;
				}

				for (const m of msg.bulk_msgs) this.handle_msg(m);
				break;
			}
			case 'eval': {
				if (msg.cell === undefined) {
					console.error('Expected cell ref for EVAL msg from server.');
					return;
				} else if (msg.eval === undefined) {
					console.error('Expected cell value for EVAL msg from server.');
					return;
				}

				let pos = new Position(msg.cell.row, msg.cell.col);
				if (this.data.cells[pos.key()] === undefined) return;

				this.data.cells[pos.key()].temp_eval = msg.eval;

				break;
			}
		}
	}
}

export { Position, Grid };
