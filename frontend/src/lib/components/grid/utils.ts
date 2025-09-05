export type CellValue = number | string | undefined;

export interface CellData {
	raw_val: string;
	val: CellValue;
}

export interface CellRef {
	row: number;
	col: number;
	str: string;
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
	return { row: row - 1, col: col - 1, str: ref };
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
export function refFromPos(row: number, col: number): CellRef {
	return { row, col, str: colToStr(col) + (row + 1).toString() };
}

export function splitErrorString(errorString: string) {
	// Remove the "ERR " prefix.
	const content = errorString.substring(4);

	// Find the index of the first colon.
	const colonIndex = content.indexOf(':');

	// If no colon is found, return the whole content as the first element.
	if (colonIndex === -1) {
		return [content.trim(), ''];
	}

	// Extract the part before the colon (the error type).
	const errorType = content.substring(0, colonIndex).trim();

	// Extract the part after the colon (the error message).
	const errorMessage = content.substring(colonIndex + 1).trim();

	return [errorType, errorMessage];
}
