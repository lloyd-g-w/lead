import { toast } from 'svelte-sonner';
import type { CellRef, CellT, Eval, LeadMsg } from './messages';
import { refToStr } from './utils';

class Position {
	public row: number;
	public col: number;

	constructor(row: number, col: number) {
		this.row = row;
		this.col = col;
	}

	public key() {
		return `${this.row}:${this.col}`;
	}

	public static key(row: number, col: number) {
		return `${row}:${col}`;
	}

	public ref(): CellRef {
		return { row: this.row, col: this.col };
	}

	public str(): string {
		return refToStr(this.row, this.col);
	}

	public equals(other: CellRef | null | undefined): boolean {
		return !!other && this.row === other.row && this.col === other.col;
	}

	public static equals(a: CellRef | null | undefined, b: CellRef | null | undefined): boolean {
		return !!a && !!b && a.row === b.row && a.col === b.col;
	}
}

class Grid {
	data: Record<string, CellT> = $state({});
	socket: WebSocket;
	row_heights: Record<number, string> = $state({});
	col_widths: Record<number, string> = $state({});
	default_row_height: string;
	default_col_width: string;
	primary_active: Position | null = $state(null);
	secondary_active: Position | null = $state(null);
	editing_cell: Position | null = $state(null);
	external_editing_cell: Position | null = $state(null);
	editing_preview: [Eval, boolean] | null = $state(null); // [Eval, dirty]

	constructor(socket: WebSocket, default_col_width = '80px', default_row_height = '30px') {
		this.socket = socket;
		this.default_col_width = default_col_width;
		this.default_row_height = default_row_height;
	}

	public getCell(pos: Position): CellT | undefined {
		return this.data[pos.key()];
	}

	public setCell(pos: Position | null | undefined) {
		if (pos === null || pos === undefined) return;
		let data = this.data[pos.key()];
		if (data === undefined) return;

		if (data.temp_raw === '') {
			delete this.data[pos.key()];
			return;
		}

		data.raw = data.temp_raw;
		data.eval = data.temp_eval;

		let msg: LeadMsg = {
			msg_type: 'set',
			cell: pos.ref(),
			raw: data.temp_raw
		};

		this.socket.send(JSON.stringify(msg));
	}

	public setCellTemp(pos: Position | null, raw: string | undefined) {
		if (pos === null || raw === undefined) return;

		let x = this.data[pos.key()];

		this.data[pos.key()] = {
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

		let x = this.data[pos.key()];

		this.data[pos.key()] = {
			raw: x?.raw ?? '',
			pos: pos,
			temp_raw: x?.raw ?? '',
			eval: x?.eval ?? undefined,
			temp_eval: undefined
		};
	}

	public clearActive(){
		this.setActive(null,null);
	}

	public getRowHeight(row: number) {
		return this.row_heights[row] ?? this.default_row_height;
	}

	public getColWidth(col: number) {
		return this.col_widths[col] ?? this.default_col_width;
	}

	public getDefaultColWidth() {
		return this.default_col_width;
	}
	public getDefaultRowHeight() {
		return this.default_row_height;
	}

	public isActiveTop(pos: Position): boolean {
		const tl = this.getActiveTopLeft();
		if (!tl) return false;
		return this.isActive(pos) && pos.row === tl.row;
	}

	public isActiveBottom(pos: Position): boolean {
		const br = this.getActiveBottomRight();
		if (!br) return false;
		return this.isActive(pos) && pos.row === br.row;
	}

	public isActiveLeft(pos: Position): boolean {
		const tl = this.getActiveTopLeft();
		if (!tl) return false;
		return this.isActive(pos) && pos.col === tl.col;
	}

	public isActiveRight(pos: Position): boolean {
		const br = this.getActiveBottomRight();
		if (!br) return false;
		return this.isActive(pos) && pos.col === br.col;
	}

	public setRowHeight(row: number, height: string) {
		if (height === this.default_row_height) {
			delete this.row_heights[row];
		} else {
			this.row_heights[row] = height;
		}
	}

	public setColWidth(col: number, width: string) {
		if (width === this.default_col_width) {
			delete this.col_widths[col];
		} else {
			this.col_widths[col] = width;
		}
	}

	public startEditing(pos: Position | undefined) {
		if (!pos) return;

		this.setActive(pos, pos);
		this.editing_cell = pos;

		let cell = this.getCell(pos);
		if (!cell) return;
		cell.temp_eval = undefined;
	}

	public stopEditing(pos: Position | null | undefined) {
		if (!pos) return;
		this.editing_cell = null;
		// this.setCell(pos);
	}

	public stopAnyEditing() {
		this.editing_cell = null;
	}

	public stopEditingActive() {
		if (!this.anyIsActive() || !this.primary_active?.equals(this.secondary_active)) return;
		this.stopEditing(this.primary_active);
	}

	public isEditing(pos: Position): boolean {
		if (this.editing_cell === null) return false;
		return this.editing_cell.equals(pos);
	}

	public anyIsEditing(): boolean {
		return this.editing_cell !== null;
	}

	public isExternalEditing(pos: Position): boolean {
		if (this.external_editing_cell === null) return false;
		return this.external_editing_cell.equals(pos);
	}

	public setActive(primary: Position | null, secondary: Position | null) {
		this.primary_active = primary;
		this.secondary_active = secondary;
	}

	public setInactive() {
		this.primary_active = null;
		this.secondary_active = null;
	}

	public startExternalEdit(pos: Position | null) {
		this.external_editing_cell = pos;
	}

	public stopExternalEdit(pos: Position | null) {
		this.external_editing_cell = null;
	}

	public getActiveCell(): CellT | undefined {
		if (this.primary_active === null || this.secondary_active === null) {
			return {
				raw: '',
				temp_raw: '',
				pos: new Position(-1, -1),
				eval: undefined
			};
		}

		if (!this.primary_active.equals(this.secondary_active)) {
			return {
				raw: '',
				temp_raw: '',
				pos: new Position(-1, -1),
				eval: undefined
			};
		}

		return this.getCell(this.primary_active);
	}

	public getActiveRangeStr(): string {
		const tl = this.getActiveTopLeft();
		const br = this.getActiveBottomRight();

		if (tl === null || br === null) return '';

		// Single-cell selection
		if (tl.equals(br)) return tl.str();

		// Range selection
		return `${tl.str()}:${br.str()}`;
	}

	public getActivePos(): Position | null {
		if (
			this.primary_active === null ||
			this.secondary_active === null ||
			!this.primary_active.equals(this.secondary_active)
		) {
			return null;
		}
		return this.primary_active;
	}

	public isActive(pos: Position): boolean {
		if (this.primary_active === null || this.secondary_active === null) return false;

		return (
			pos.row >= Math.min(this.primary_active.row, this.secondary_active.row) &&
			pos.row <= Math.max(this.primary_active.row, this.secondary_active.row) &&
			pos.col >= Math.min(this.primary_active.col, this.secondary_active.col) &&
			pos.col <= Math.max(this.primary_active.col, this.secondary_active.col)
		);
	}

	public getActiveTopLeft(): Position | null {
		if (this.primary_active === null || this.secondary_active === null) return null;

		return new Position(
			Math.min(this.primary_active.row, this.secondary_active.row),
			Math.min(this.primary_active.col, this.secondary_active.col)
		);
	}

	public getActiveBottomRight(): Position | null {
		if (this.primary_active === null || this.secondary_active === null) return null;

		return new Position(
			Math.max(this.primary_active.row, this.secondary_active.row),
			Math.max(this.primary_active.col, this.secondary_active.col)
		);
	}

	public isPrimaryActive(pos: Position): boolean {
		if (this.primary_active === null) return false;
		return this.primary_active.equals(pos);
	}

	public isSingleActive(): boolean {
		return this.getActivePos() !== null;
	}

	public anyIsActive(): boolean {
		return this.primary_active !== null && this.secondary_active !== null;
	}

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

				let x = this.data[pos.key()];

				this.data[pos.key()] = {
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
				if (this.data[pos.key()] === undefined) return;

				this.data[pos.key()].temp_eval = msg.eval;

				break;
			}
		}
	}
}

export { Position, Grid };
