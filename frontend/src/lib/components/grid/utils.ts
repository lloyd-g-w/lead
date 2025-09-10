import type { CellRef, Eval, LiteralValue } from './messages';

/**
 * Zero indexed | A1 == {row: 0, col: 0};
 */
export function refFromStr(ref: string): CellRef {
	const match = ref.match(/^([A-Z]+)([0-9]+)$/i);
	if (!match) throw new Error('Invalid reference');

	const [, letters, rowStr] = match;

	let col = 0;
	for (let i = 0; i < letters.length; i++) {
		col = col * 26 + (letters.charCodeAt(i) - 64); // 'A' = 65 → 1
	}

	const row = parseInt(rowStr, 10);
	return { row: row - 1, col: col - 1 };
}

/**
 * Zero indexed | 0 == A;
 */
export function colToStr(col: number): string {
	let result = '';
	let n = col;
	while (n >= 0) {
		const rem = n % 26;
		result = String.fromCharCode(65 + rem) + result; // 65 = 'A'
		n = Math.floor(n / 26) - 1;
	}

	return result;
}

/**
 * Zero indexed | A1 == {row: 0, col: 0};
 */
export function refToStr(row: number, col: number): string {
	return colToStr(col) + (row + 1).toString();
}

export function getEvalLiteral(value: Eval | undefined): LiteralValue {
	if (value === undefined) return '';
	if (value === 'unset') return '';
	if ('literal' in value) {
		if (value.literal.value == null) return 'NaN';
		return value.literal.value;
	}
	if ('cellref' in value) return getEvalLiteral(value.cellref.eval);
	if ('err' in value) return `#${value.err.code.toUpperCase()}`;
	// if ('range' in value) return 'err';
	return 'todo!';
}

export function isErr(value: Eval | undefined): boolean {
	if (value === undefined) return false;
	if (value === 'unset') return false;
	if ('cellref' in value) return isErr(value.cellref.eval);
	return 'err' in value;
}

export function getErrTitle(value: Eval | undefined): string {
	if (value === undefined) return '';
	if (value === 'unset') return '';
	if ('cellref' in value) return getErrTitle(value.cellref.eval);
	if (!('err' in value)) return '';
	return value.err.title;
}

export function getErrDesc(value: Eval | undefined): string {
	if (value === undefined) return '';
	if (value === 'unset') return '';
	if ('cellref' in value) return getErrDesc(value.cellref.eval);
	if (!('err' in value)) return '';
	return value.err.desc;
}
