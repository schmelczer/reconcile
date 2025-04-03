import { positionToLineAndColumn } from "./position-to-line-and-column";

describe("positionToLineAndColumn", () => {
	test("converts position to line and column in multi-line text", () => {
		const text = "ab\ncd\n";
		expect(positionToLineAndColumn(text, 0)).toEqual({
			line: 0,
			column: 0
		});
		expect(positionToLineAndColumn(text, 1)).toEqual({
			line: 0,
			column: 1
		});
		expect(positionToLineAndColumn(text, 2)).toEqual({
			line: 0,
			column: 2
		});
		expect(positionToLineAndColumn(text, 3)).toEqual({
			line: 1,
			column: 0
		});
		expect(positionToLineAndColumn(text, 4)).toEqual({
			line: 1,
			column: 1
		});
		expect(positionToLineAndColumn(text, 6)).toEqual({
			line: 2,
			column: 0
		});
	});

	test("with carrige returns", () => {
		expect(positionToLineAndColumn("a\nb", 3)).toEqual({
			line: 1,
			column: 1
		});

		expect(positionToLineAndColumn("a\r\nb", 3)).toEqual({
			line: 1,
			column: 1
		});
	});

	test("handles empty input", () => {
		expect(positionToLineAndColumn("", 0)).toEqual({ line: 0, column: 0 });
	});

	test("handles positions at the end of text", () => {
		const text = "End";
		expect(positionToLineAndColumn(text, 3)).toEqual({
			line: 0,
			column: 3
		});
	});

	test("throws error for position out of range", () => {
		const text = "Short text";
		expect(() => positionToLineAndColumn(text, 15)).toThrow();
		expect(() => positionToLineAndColumn(text, -1)).toThrow();
	});
});
