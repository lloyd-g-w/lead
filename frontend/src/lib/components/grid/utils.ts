export function fromGridRef(ref: string): [number, number] {
	const match = ref.match(/^([A-Z]+)([0-9]+)$/i);
	if (!match) throw new Error('Invalid reference');

	const [, letters, rowStr] = match;

	let col = 0;
	for (let i = 0; i < letters.length; i++) {
		col = col * 26 + (letters.charCodeAt(i) - 64); // 'A' = 65 → 1
	}

	const row = parseInt(rowStr, 10);
	return [row - 1, col - 1];
}

export function toColLetter(col: number): string {
	let result = '';
	let n = col;
	while (n > 0) {
		const rem = (n - 1) % 26;
		result = String.fromCharCode(65 + rem) + result; // 65 = 'A'
		n = Math.floor((n - 1) / 26);
	}

	return result;
}

export function toGridRef(row: number, col: number): string {
	row++;
	col++;
	return toColLetter(col) + row.toString();
}

export type CellValue = number | string | undefined;

export interface CellData {
	raw_val: string;
	val: CellValue;
}
