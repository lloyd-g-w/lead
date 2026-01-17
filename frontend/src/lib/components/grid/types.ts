import type { CellT } from './messages';

export interface GridDefaults {
	row_height: number;
	col_width: number;
}

export interface GridData {
	cells: Record<string, CellT>;
	row_heights: Record<number, number>;
	col_widths: Record<number, number>;
}
