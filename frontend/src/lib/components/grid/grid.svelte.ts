import { toast } from 'svelte-sonner';
import type { CellRef, CellT, Eval, LeadMsg } from './messages';

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
	active_cell: Position | null = $state(null);
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

	public setCell(pos: Position | null) {
		if (pos === null) return;
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
			eval: x?.eval ?? undefined,
			temp_eval: x?.temp_eval ?? undefined
		};

		this.quickEval(pos, raw);
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

	public startEditing(pos: Position) {
		this.active_cell = pos;
		this.editing_cell = pos;

		let cell = this.getCell(pos);
		if (!cell) return;
		cell.temp_eval = undefined;
	}

	public stopEditing(pos: Position) {
		this.editing_cell = null;
		this.setCell(pos);
	}

	public stopEditingActive() {
		if (this.active_cell == null) return;
		this.stopEditing(this.active_cell);
	}

	public isEditing(pos: Position): boolean {
		if (this.editing_cell == null) return false;
		return this.editing_cell.equals(pos);
	}

	public isExternalEditing(pos: Position): boolean {
		if (this.external_editing_cell == null) return false;
		return this.external_editing_cell.equals(pos);
	}

	public setActive(pos: Position | null) {
		this.active_cell = pos;
	}

	public setExternalEdit(pos: Position | null) {
		this.external_editing_cell = pos;
	}

	public getActiveCell(): CellT | undefined {
		if (this.active_cell === null)
			return {
				raw: '',
				temp_raw: '',
				eval: undefined
			};

		return this.getCell(this.active_cell);
	}

	public getActivePos(): Position | null {
		return this.active_cell;
	}

	public isActive(pos: Position): boolean {
		if (this.active_cell == null) return false;
		return this.active_cell.equals(pos);
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
