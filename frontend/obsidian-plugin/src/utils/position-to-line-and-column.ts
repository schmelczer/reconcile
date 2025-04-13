/**
 * Converts a character position in text to line and column numbers.
 *
 * @param text The text content to analyze
 * @param position The character position to convert
 * @returns An object containing line and column numbers
 * @throws Will throw an error if the position is negative or exceeds the text length
 */
export function positionToLineAndColumn(
	text: string,
	position: number
): { line: number; column: number } {
	if (position < 0) {
		throw new Error("Position cannot be negative");
	}

	text = text.replace("\r", "");

	if (
		position >
		text.length + 1
		// +1 to account for the cursor being after last character
	) {
		throw new Error(
			`Position ${position} exceeds text length ${text.length}`
		);
	}

	const textUpToPosition = text.substring(0, position);
	const lines = textUpToPosition.split("\n");

	const line = lines.length - 1;
	const column = lines[lines.length - 1].length;

	return { line, column };
}
