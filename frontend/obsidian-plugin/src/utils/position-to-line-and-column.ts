/**
 * Converts a character position in text to line and column numbers.
 *
 * @param text The text content to analyze
 * @param position The character position to convert
 * @returns An object containing line and column numbers (0-based index for line, 1-based index for column)
 * @throws Will throw an error if the position is negative or exceeds the text length
 */
export function positionToLineAndColumn(
	text: string,
	position: number
): { line: number; column: number } {
	if (position < 0) {
		throw new Error("Position cannot be negative");
	}

	if (position > text.length) {
		throw new Error(
			`Position ${position} exceeds text length ${text.length}`
		);
	}

	const textUpToPosition = text.substring(0, position);
	const lines = textUpToPosition.split("\n");

	const line = lines.length - 1; // 0-based index
	const column = lines[lines.length - 1].length + 1; // 1-based index

	return { line, column };
}
