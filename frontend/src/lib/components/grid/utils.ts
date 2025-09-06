export interface CellT {
	raw_val: string;
	val: LiteralValue | undefined;
}

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
