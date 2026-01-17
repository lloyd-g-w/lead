import type { CellRef } from './messages';
import { refToStr } from './utils';

export class Position {
	public row: number = $state(0);
	public col: number = $state(0);

	constructor(row: number = 0, col: number = 0) {
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
