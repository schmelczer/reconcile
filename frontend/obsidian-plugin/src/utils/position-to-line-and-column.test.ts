import { positionToLineAndColumn } from "./position-to-line-and-column";

describe("positionToLineAndColumn", () => {
	test("converts position to line and column in a single line text", () => {
		const text = "Hello, world!";
		expect(positionToLineAndColumn(text, 0)).toEqual({
			line: 0,
			column: 1
		});
		expect(positionToLineAndColumn(text, 7)).toEqual({
			line: 0,
			column: 8
		});
		expect(positionToLineAndColumn(text, 12)).toEqual({
			line: 0,
			column: 13
		});
	});

	test("converts position to line and column in multi-line text", () => {
		const text = "First line\nSecond line\nThird line";
		expect(positionToLineAndColumn(text, 0)).toEqual({
			line: 0,
			column: 1
		});
		expect(positionToLineAndColumn(text, 10)).toEqual({
			line: 0,
			column: 11
		});
		expect(positionToLineAndColumn(text, 15)).toEqual({
			line: 1,
			column: 5
		});
		expect(positionToLineAndColumn(text, 26)).toEqual({
			line: 2,
			column: 4
		});
	});

	test("handles positions at line breaks", () => {
		const text = "Line\nBreak";
		expect(positionToLineAndColumn(text, 4)).toEqual({
			line: 0,
			column: 5
		});
		expect(positionToLineAndColumn(text, 5)).toEqual({
			line: 1,
			column: 1
		});
	});

	test("handles empty input", () => {
		expect(positionToLineAndColumn("", 0)).toEqual({ line: 0, column: 1 });
	});

	test("handles positions at the end of text", () => {
		const text = "End";
		expect(positionToLineAndColumn(text, 3)).toEqual({
			line: 0,
			column: 4
		});
	});

	test("throws error for position out of range", () => {
		const text = "Short text";
		expect(() => positionToLineAndColumn(text, 15)).toThrow();
		expect(() => positionToLineAndColumn(text, -1)).toThrow();
	});
});
