/**
 * Converts line and column coordinates to an absolute character position in a text string.
 *
 * @param line - The zero-based line number
 * @param column - The zero-based column number
 * @param text - The text string to calculate position in
 * @returns The absolute character position (zero-based index) in the text string
 * @throws Error if line number is out of range
 * @throws Error if column number is out of range
 */
export function lineAndColumnToPosition(
	text: string,
	line: number,
	column: number
): number {
	const lines = text.split("\n");

	if (line >= lines.length) {
		throw new Error(`Line number ${line} is out of range.`);
	}

	if (column > lines[line].length) {
		throw new Error(`Column number ${column} is out of range.`);
	}

	let position = 0;
	for (let i = 0; i < line; i++) {
		position += lines[i].length + 1;
	}

	position += column;

	return position;
}
